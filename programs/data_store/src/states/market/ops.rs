use anchor_lang::prelude::*;

use anchor_spl::token::Mint;
use gmx_core::{
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::PositionImpactDistributionParams,
        FeeParams, PositionParams, PriceImpactParams,
    },
    BaseMarketExt, ClockKind, PoolKind,
};

use crate::{
    constants,
    states::{
        position::{Position, PositionOps},
        Store,
    },
    utils::internal::TransferUtils,
    DataStoreError, GmxCoreError,
};

use super::{config::MarketConfig, Clocks, HasMarketMeta, Market, MarketMeta, Pool, Pools};

/// Convert to a [`Market`](gmx_core::Market).
pub struct AsMarket<'a, 'info> {
    is_pure: bool,
    meta: &'a MarketMeta,
    config: &'a MarketConfig,
    long_token_balance: &'a u64,
    short_token_balance: &'a u64,
    funding_factor_per_second: &'a mut i128,
    pools: &'a mut Pools,
    clocks: &'a mut Clocks,
    mint: &'a mut Account<'info, Mint>,
    transfer: Option<TransferUtils<'a, 'info>>,
    receiver: Option<AccountInfo<'info>>,
    vault: Option<AccountInfo<'info>>,
}

impl<'a, 'info> AsMarket<'a, 'info> {
    pub(super) fn new(market: &'a mut Market, mint: &'a mut Account<'info, Mint>) -> Self {
        Self {
            is_pure: market.is_pure(),
            meta: &market.meta,
            config: &market.config,
            long_token_balance: &market.state.long_token_balance,
            short_token_balance: &market.state.short_token_balance,
            pools: &mut market.pools,
            clocks: &mut market.clocks,
            mint,
            transfer: None,
            receiver: None,
            vault: None,
            funding_factor_per_second: &mut market.state.funding_factor_per_second,
        }
    }

    pub(crate) fn enable_transfer(
        mut self,
        token_program: AccountInfo<'info>,
        store: &'a AccountLoader<'info, Store>,
    ) -> Self {
        self.transfer = Some(TransferUtils::new(
            token_program,
            store,
            Some(self.mint.to_account_info()),
        ));
        self
    }

    pub(crate) fn with_receiver(mut self, receiver: AccountInfo<'info>) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub(crate) fn with_vault(mut self, vault: AccountInfo<'info>) -> Self {
        self.vault = Some(vault);
        self
    }

    pub(crate) fn meta(&self) -> &MarketMeta {
        self.meta
    }

    pub(crate) fn into_position_ops(
        self,
        position: &'a mut AccountLoader<'info, Position>,
    ) -> Result<PositionOps<'a, 'info>> {
        PositionOps::try_new(self, position)
    }

    fn balance(&self, is_long_token: bool) -> u64 {
        if is_long_token || self.is_pure {
            if self.is_pure {
                *self.long_token_balance / 2
            } else {
                *self.long_token_balance
            }
        } else {
            *self.short_token_balance
        }
    }

    fn balance_after_excluding(&self, is_long_token: bool, excluding_amount: u64) -> Result<u64> {
        let balance = self.balance(is_long_token);
        if excluding_amount != 0 {
            balance
                .checked_sub(excluding_amount)
                .ok_or(error!(DataStoreError::AmountOverflow))
        } else {
            Ok(balance)
        }
    }

    fn adjust_excluding_amounts(
        &self,
        long_excluding_amount: u64,
        short_excluding_amount: u64,
    ) -> Result<(u64, u64)> {
        if self.is_pure {
            let total = long_excluding_amount
                .checked_add(short_excluding_amount)
                .ok_or(DataStoreError::AmountOverflow)?;
            Ok((total / 2, total / 2))
        } else {
            Ok((long_excluding_amount, short_excluding_amount))
        }
    }

    pub(crate) fn validate_market_balances(
        &self,
        long_excluding_amount: u64,
        short_excluding_amount: u64,
    ) -> Result<()> {
        let (long_excluding_amount, short_excluding_amount) =
            self.adjust_excluding_amounts(long_excluding_amount, short_excluding_amount)?;
        let long_token_balance = self.balance_after_excluding(true, long_excluding_amount)? as u128;
        self.validate_token_balance_for_one_side(&long_token_balance, true)
            .map_err(GmxCoreError::from)?;
        let short_token_balance =
            self.balance_after_excluding(false, short_excluding_amount)? as u128;
        self.validate_token_balance_for_one_side(&short_token_balance, false)
            .map_err(GmxCoreError::from)?;
        Ok(())
    }

    pub(crate) fn validate_market_balances_excluding_token_amount(
        &self,
        token: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        if *token == self.meta.long_token_mint {
            self.validate_market_balances(amount, 0)
        } else if *token == self.meta.short_token_mint {
            self.validate_market_balances(0, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }

    fn get_pool(&self, kind: PoolKind) -> gmx_core::Result<&Pool> {
        self.pools
            .get(kind)
            .ok_or(gmx_core::Error::MissingPoolKind(kind))
    }

    fn get_pool_mut(&mut self, kind: PoolKind) -> gmx_core::Result<&mut Pool> {
        self.pools
            .get_mut(kind)
            .ok_or(gmx_core::Error::MissingPoolKind(kind))
    }

    fn just_passed_in_seconds(&mut self, clock: ClockKind) -> gmx_core::Result<u64> {
        let current = Clock::get().map_err(Error::from)?.unix_timestamp;
        let last = self
            .clocks
            .get_mut(clock)
            .ok_or(gmx_core::Error::MissingClockKind(clock))?;
        let duration = current.saturating_sub(*last);
        if duration > 0 {
            *last = current;
        }
        Ok(duration as u64)
    }
}

impl<'a, 'info> gmx_core::BaseMarket<{ constants::MARKET_DECIMALS }> for AsMarket<'a, 'info> {
    type Num = u128;

    type Signed = i128;

    type Pool = Pool;

    fn liquidity_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(PoolKind::Primary)
    }

    fn liquidity_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(PoolKind::Primary)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        constants::MARKET_USD_TO_AMOUNT_DIVISOR
    }

    fn reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        Ok(self.config.reserve_factor)
    }

    fn max_pnl_factor(
        &self,
        kind: gmx_core::PnlFactorKind,
        is_long: bool,
    ) -> gmx_core::Result<Self::Num> {
        use gmx_core::PnlFactorKind;

        match (kind, is_long) {
            (PnlFactorKind::Deposit, true) => Ok(self.config.max_pnl_factor_for_long_deposit),
            (PnlFactorKind::Deposit, false) => Ok(self.config.max_pnl_factor_for_short_deposit),
            (PnlFactorKind::Withdrawal, true) => Ok(self.config.max_pnl_factor_for_long_withdrawal),
            (PnlFactorKind::Withdrawal, false) => {
                Ok(self.config.max_pnl_factor_for_short_withdrawal)
            }
            _ => Err(error!(DataStoreError::RequiredResourceNotFound).into()),
        }
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmx_core::Result<Self::Num> {
        if is_long_token {
            Ok(self.config.max_pool_amount_for_long_token)
        } else {
            Ok(self.config.max_pool_amount_for_short_token)
        }
    }

    fn claimable_fee_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(PoolKind::ClaimableFee)
    }

    fn claimable_fee_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(PoolKind::ClaimableFee)
    }

    fn swap_impact_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(PoolKind::SwapImpact)
    }

    fn open_interest_pool(&self, is_long: bool) -> gmx_core::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        };
        self.get_pool(kind)
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmx_core::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        };
        self.get_pool(kind)
    }
}

impl<'a, 'info> gmx_core::SwapMarket<{ constants::MARKET_DECIMALS }> for AsMarket<'a, 'info> {
    fn swap_impact_params(&self) -> gmx_core::Result<PriceImpactParams<Self::Num>> {
        PriceImpactParams::builder()
            .with_exponent(self.config.swap_impact_exponent)
            .with_positive_factor(self.config.swap_impact_positive_factor)
            .with_negative_factor(self.config.swap_impact_negative_factor)
            .build()
    }

    fn swap_fee_params(&self) -> gmx_core::Result<FeeParams<Self::Num>> {
        Ok(FeeParams::builder()
            .with_fee_receiver_factor(self.config.swap_fee_receiver_factor)
            .with_positive_impact_fee_factor(self.config.swap_fee_factor_for_positive_impact)
            .with_negative_impact_fee_factor(self.config.swap_fee_factor_for_positive_impact)
            .build())
    }

    fn swap_impact_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(PoolKind::SwapImpact)
    }
}

impl<'a, 'info> gmx_core::PositionImpactMarket<{ constants::MARKET_DECIMALS }>
    for AsMarket<'a, 'info>
{
    fn position_impact_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(PoolKind::PositionImpact)
    }

    fn position_impact_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(PoolKind::PositionImpact)
    }

    fn just_passed_in_seconds_for_position_impact_distribution(&mut self) -> gmx_core::Result<u64> {
        self.just_passed_in_seconds(ClockKind::PriceImpactDistribution)
    }

    fn position_impact_params(&self) -> gmx_core::Result<PriceImpactParams<Self::Num>> {
        PriceImpactParams::builder()
            .with_exponent(self.config.position_impact_exponent)
            .with_positive_factor(self.config.position_impact_positive_factor)
            .with_negative_factor(self.config.position_impact_negative_factor)
            .build()
    }

    fn position_impact_distribution_params(
        &self,
    ) -> gmx_core::Result<PositionImpactDistributionParams<Self::Num>> {
        Ok(PositionImpactDistributionParams::builder()
            .distribute_factor(self.config.position_impact_distribute_factor)
            .min_position_impact_pool_amount(self.config.min_position_impact_pool_amount)
            .build())
    }
}

impl<'a, 'info> gmx_core::LiquidityMarket<{ constants::MARKET_DECIMALS }> for AsMarket<'a, 'info> {
    fn total_supply(&self) -> Self::Num {
        self.mint.supply.into()
    }

    fn mint(&mut self, amount: &Self::Num) -> gmx_core::Result<()> {
        let Some(transfer) = self.transfer.as_ref() else {
            return Err(gmx_core::Error::invalid_argument("transfer not enabled"));
        };
        let Some(receiver) = self.receiver.as_ref() else {
            return Err(gmx_core::Error::MintReceiverNotSet);
        };
        transfer.mint_to(
            receiver,
            (*amount)
                .try_into()
                .map_err(|_| gmx_core::Error::Overflow)?,
        )?;
        self.mint.reload()?;
        Ok(())
    }

    fn burn(&mut self, amount: &Self::Num) -> gmx_core::Result<()> {
        let Some(transfer) = self.transfer.as_ref() else {
            return Err(gmx_core::Error::invalid_argument("transfer not enabled"));
        };
        let Some(vault) = self.vault.as_ref() else {
            return Err(gmx_core::Error::WithdrawalVaultNotSet);
        };
        transfer.burn_from(
            vault,
            (*amount)
                .try_into()
                .map_err(|_| gmx_core::Error::Overflow)?,
        )?;
        self.mint.reload()?;
        Ok(())
    }

    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> gmx_core::Result<Self::Num> {
        if is_long_token {
            Ok(self.config.max_pool_value_for_deposit_for_long_token)
        } else {
            Ok(self.config.max_pool_value_for_deposit_for_short_token)
        }
    }
}

impl<'a, 'info> gmx_core::PerpMarket<{ constants::MARKET_DECIMALS }> for AsMarket<'a, 'info> {
    fn funding_amount_per_size_adjustment(&self) -> Self::Num {
        constants::FUNDING_AMOUNT_PER_SIZE_ADJUSTMENT
    }

    fn position_params(&self) -> gmx_core::Result<PositionParams<Self::Num>> {
        // TODO: use min collateral factors for OI.
        Ok(PositionParams::new(
            self.config.min_position_size_usd,
            self.config.min_collateral_value,
            self.config.min_collateral_factor,
            self.config.max_positive_position_impact_factor,
            self.config.max_negative_position_impact_factor,
            self.config.max_position_impact_factor_for_liquidations,
        ))
    }

    fn order_fee_params(&self) -> gmx_core::Result<FeeParams<Self::Num>> {
        Ok(FeeParams::builder()
            .with_fee_receiver_factor(self.config.order_fee_receiver_factor)
            .with_positive_impact_fee_factor(self.config.order_fee_factor_for_positive_impact)
            .with_negative_impact_fee_factor(self.config.order_fee_factor_for_negative_impact)
            .build())
    }

    fn borrowing_fee_params(&self) -> gmx_core::Result<BorrowingFeeParams<Self::Num>> {
        Ok(BorrowingFeeParams::builder()
            .receiver_factor(self.config.borrowing_fee_receiver_factor)
            .factor_for_long(self.config.borrowing_fee_factor_for_long)
            .factor_for_short(self.config.borrowing_fee_factor_for_short)
            .exponent_for_long(self.config.borrowing_fee_exponent_for_long)
            .exponent_for_short(self.config.borrowing_fee_exponent_for_short)
            .build())
    }

    fn funding_fee_params(&self) -> gmx_core::Result<FundingFeeParams<Self::Num>> {
        Ok(FundingFeeParams::builder()
            .exponent(self.config.funding_fee_exponent)
            .funding_factor(self.config.funding_fee_factor)
            .max_factor_per_second(self.config.funding_fee_max_factor_per_second)
            .min_factor_per_second(self.config.funding_fee_min_factor_per_second)
            .increase_factor_per_second(self.config.funding_fee_increase_factor_per_second)
            .decrease_factor_per_second(self.config.funding_fee_decrease_factor_per_second)
            .threshold_for_stable_funding(self.config.funding_fee_threshold_for_stable_funding)
            .threshold_for_decrease_funding(self.config.funding_fee_threshold_for_decrease_funding)
            .build())
    }

    fn funding_factor_per_second(&self) -> &Self::Signed {
        self.funding_factor_per_second
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        self.funding_factor_per_second
    }

    fn open_interest_reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        Ok(self.config.open_interest_reserve_factor)
    }

    fn max_open_interest(&self, is_long: bool) -> gmx_core::Result<Self::Num> {
        if is_long {
            Ok(self.config.max_open_interest_for_long)
        } else {
            Ok(self.config.max_open_interest_for_short)
        }
    }

    fn open_interest_pool_mut(&mut self, is_long: bool) -> gmx_core::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        };
        self.get_pool_mut(kind)
    }

    fn open_interest_in_tokens_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmx_core::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        };
        self.get_pool_mut(kind)
    }

    fn borrowing_factor_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(PoolKind::BorrowingFactor)
    }

    fn borrowing_factor_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(PoolKind::BorrowingFactor)
    }

    fn funding_amount_per_size_pool(&self, is_long: bool) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        })
    }

    fn funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        })
    }

    fn claimable_funding_amount_per_size_pool(
        &self,
        is_long: bool,
    ) -> gmx_core::Result<&Self::Pool> {
        self.get_pool(if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        })
    }

    fn claimable_funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmx_core::Result<&mut Self::Pool> {
        self.get_pool_mut(if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        })
    }

    fn just_passed_in_seconds_for_borrowing(&mut self) -> gmx_core::Result<u64> {
        self.just_passed_in_seconds(ClockKind::Borrowing)
    }

    fn just_passed_in_seconds_for_funding(&mut self) -> gmx_core::Result<u64> {
        self.just_passed_in_seconds(ClockKind::Funding)
    }
}

/// Extension trait for validating market balances.
pub trait ValidateMarketBalances:
    gmx_core::BaseMarket<{ constants::MARKET_DECIMALS }, Num = u128>
    + gmx_core::Bank<Pubkey, Num = u64>
    + HasMarketMeta
{
    /// Validate market balances.
    fn validate_market_balances(
        &self,
        mut long_excluding_amount: u64,
        mut short_excluding_amount: u64,
    ) -> Result<()> {
        if self.is_pure() {
            let total = long_excluding_amount
                .checked_add(short_excluding_amount)
                .ok_or(DataStoreError::AmountOverflow)?;
            (long_excluding_amount, short_excluding_amount) = (total / 2, total / 2);
        }
        let meta = self.market_meta();
        let long_token_balance = self
            .balance_excluding(&meta.long_token_mint, &long_excluding_amount)
            .map_err(GmxCoreError::from)?
            .into();
        self.validate_token_balance_for_one_side(&long_token_balance, true)
            .map_err(GmxCoreError::from)?;
        let short_token_balance = self
            .balance_excluding(&meta.short_token_mint, &short_excluding_amount)
            .map_err(GmxCoreError::from)?
            .into();
        self.validate_token_balance_for_one_side(&short_token_balance, false)
            .map_err(GmxCoreError::from)?;
        Ok(())
    }

    /// Validate market balance for the given token.
    fn validate_market_balances_excluding_token_amount(
        &self,
        token: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        let side = self.market_meta().to_token_side(token)?;
        let balance = self
            .balance_excluding(token, &amount)
            .map_err(GmxCoreError::from)?
            .into();
        self.validate_token_balance_for_one_side(&balance, side)
            .map_err(GmxCoreError::from)?;
        Ok(())
    }
}

impl<
        M: gmx_core::BaseMarket<{ constants::MARKET_DECIMALS }, Num = u128>
            + gmx_core::Bank<Pubkey, Num = u64>
            + HasMarketMeta,
    > ValidateMarketBalances for M
{
}
