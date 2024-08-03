use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gmsol_model::{
    params::{position::PositionImpactDistributionParams, PriceImpactParams},
    ClockKind, PoolKind,
};

use crate::{
    constants,
    states::{clock::AsClock, HasMarketMeta, Market, Store},
    utils::internal::TransferUtils,
};

use super::{swap_market::RevertibleSwapMarket, Revertible, RevertibleMarket, RevertiblePool};

/// Convert a [`RevertibleMarket`] to a [`LiquidityMarket`](gmsol_model::LiquidityMarket).
pub struct RevertibleLiquidityMarket<'a, 'info> {
    market: RevertibleSwapMarket<'a>,
    market_token: &'a mut Account<'info, Mint>,
    transfer: TransferUtils<'a, 'info>,
    receiver: Option<AccountInfo<'info>>,
    vault: Option<AccountInfo<'info>>,
    position_impact: RevertiblePool,
    position_impact_distribution_clock: i64,
    to_mint: u64,
    to_burn: u64,
}

impl<'a, 'info> Key for RevertibleLiquidityMarket<'a, 'info> {
    fn key(&self) -> Pubkey {
        self.market.key()
    }
}

impl<'a, 'info> HasMarketMeta for RevertibleLiquidityMarket<'a, 'info> {
    fn is_pure(&self) -> bool {
        self.market.is_pure()
    }

    fn market_meta(&self) -> &crate::states::MarketMeta {
        self.market.market_meta()
    }
}

impl<'a, 'info> gmsol_model::Bank<Pubkey> for RevertibleLiquidityMarket<'a, 'info> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market.record_transferred_in_by_token(token, amount)
    }

    fn record_transferred_out_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market.record_transferred_out_by_token(token, amount)
    }

    fn balance<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &self,
        token: &Q,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.balance(token)
    }
}

impl<'a, 'info> RevertibleLiquidityMarket<'a, 'info> {
    pub(crate) fn new(
        loader: &'a AccountLoader<'info, Market>,
        market_token: &'a mut Account<'info, Mint>,
        token_program: AccountInfo<'info>,
        store: &'a AccountLoader<'info, Store>,
    ) -> Result<Self> {
        let market = loader.try_into()?;
        Self::from_market(market, market_token, token_program, store)
    }

    pub(crate) fn from_market(
        market: RevertibleMarket<'a>,
        market_token: &'a mut Account<'info, Mint>,
        token_program: AccountInfo<'info>,
        store: &'a AccountLoader<'info, Store>,
    ) -> Result<Self> {
        let position_impact = market.create_revertible_pool(PoolKind::PositionImpact)?;
        let position_impact_distribution_clock =
            market.get_clock(ClockKind::PriceImpactDistribution)?;
        Ok(Self {
            market: RevertibleSwapMarket::from_market(market)?,
            transfer: TransferUtils::new(
                token_program,
                store,
                Some(market_token.to_account_info()),
            ),
            market_token,
            receiver: None,
            vault: None,
            position_impact,
            position_impact_distribution_clock,
            to_mint: 0,
            to_burn: 0,
        })
    }

    pub(crate) fn enable_mint(mut self, receiver: AccountInfo<'info>) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub(crate) fn enable_burn(mut self, vault: AccountInfo<'info>) -> Self {
        self.vault = Some(vault);
        self
    }
}

impl<'a, 'info> gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.liquidity_pool()
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.claimable_fee_pool()
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.swap_impact_pool()
    }

    fn open_interest_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.market.open_interest_pool(is_long)
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.market.open_interest_in_tokens_pool(is_long)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.market.usd_to_amount_divisor()
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        self.market.max_pool_amount(is_long_token)
    }

    fn pnl_factor_config(
        &self,
        kind: gmsol_model::PnlFactorKind,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.pnl_factor_config(kind, is_long)
    }

    fn reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        self.market.reserve_factor()
    }
}

impl<'a, 'info> gmsol_model::BaseMarketMut<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn liquidity_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.liquidity_pool_mut()
    }

    fn claimable_fee_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.claimable_fee_pool_mut()
    }
}

impl<'a, 'info> gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn swap_impact_params(
        &self,
    ) -> gmsol_model::Result<gmsol_model::params::PriceImpactParams<Self::Num>> {
        self.market.swap_impact_params()
    }

    fn swap_fee_params(&self) -> gmsol_model::Result<gmsol_model::params::FeeParams<Self::Num>> {
        self.market.swap_fee_params()
    }
}

impl<'a, 'info> gmsol_model::SwapMarketMut<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn swap_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.swap_impact_pool_mut()
    }
}

impl<'a, 'info> gmsol_model::LiquidityMarket<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn total_supply(&self) -> Self::Num {
        self.market_token.supply.into()
    }

    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        if is_long_token {
            Ok(self
                .market
                .market
                .config()
                .max_pool_value_for_deposit_for_long_token)
        } else {
            Ok(self
                .market
                .market
                .config()
                .max_pool_value_for_deposit_for_short_token)
        }
    }

    fn mint(&mut self, amount: &Self::Num) -> gmsol_model::Result<()> {
        let new_mint: u64 = (*amount)
            .try_into()
            .map_err(|_| gmsol_model::Error::Overflow)?;
        let to_mint = self
            .to_mint
            .checked_add(new_mint)
            .ok_or(gmsol_model::Error::Overflow)?;
        // CHECK for overflow.
        self.market_token
            .supply
            .checked_add(to_mint)
            .ok_or(gmsol_model::Error::Overflow)?;
        self.to_mint = to_mint;
        Ok(())
    }

    fn burn(&mut self, amount: &Self::Num) -> gmsol_model::Result<()> {
        let new_burn: u64 = (*amount)
            .try_into()
            .map_err(|_| gmsol_model::Error::Overflow)?;
        let to_burn = self
            .to_burn
            .checked_add(new_burn)
            .ok_or(gmsol_model::Error::Overflow)?;
        // CHECK for underflow.
        self.market_token
            .supply
            .checked_sub(to_burn)
            .ok_or(gmsol_model::Error::Computation(
                "not enough market tokens to burn",
            ))?;
        self.to_burn = to_burn;
        Ok(())
    }
}

impl<'a, 'info> gmsol_model::PositionImpactMarket<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn position_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.position_impact)
    }

    fn position_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Self::Num>> {
        self.market.market.position_impact_params()
    }

    fn position_impact_distribution_params(
        &self,
    ) -> gmsol_model::Result<PositionImpactDistributionParams<Self::Num>> {
        self.market.market.position_impact_distribution_params()
    }
}

impl<'a, 'info> gmsol_model::PositionImpactMarketMut<{ constants::MARKET_DECIMALS }>
    for RevertibleLiquidityMarket<'a, 'info>
{
    fn position_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.position_impact)
    }

    fn just_passed_in_seconds_for_position_impact_distribution(
        &mut self,
    ) -> gmsol_model::Result<u64> {
        AsClock::from(&mut self.position_impact_distribution_clock).just_passed_in_seconds()
    }
}

impl<'a, 'info> Revertible for RevertibleLiquidityMarket<'a, 'info> {
    fn commit(self) {
        if self.to_mint != 0 {
            self.transfer
                .mint_to(&self.receiver.expect("mint is not enabled"), self.to_mint)
                .map_err(|err| panic!("mint error: {err}"))
                .unwrap();
        }
        if self.to_burn != 0 {
            self.transfer
                .burn_from(&self.vault.expect("burn is not enabled"), self.to_burn)
                .map_err(|err| panic!("burn error: {err}"))
                .unwrap();
        }
        self.market.commit_with(|market| {
            let position_impact = market
                .pools
                .get_mut(PoolKind::PositionImpact)
                .expect("must exist");
            self.position_impact
                .as_small_pool()
                .write_to_pool(position_impact);
            *market
                .clocks
                .get_mut(ClockKind::PriceImpactDistribution)
                .expect("must exist") = self.position_impact_distribution_clock;
        });
    }
}
