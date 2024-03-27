use std::collections::BTreeMap;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use data_store::{
    constants,
    cpi::accounts::{ApplyDeltaToMarketPool, GetPool, MintMarketTokenTo},
    states::Pool,
    utils::Authentication,
};
use gmx_core::{
    params::{FeeParams, SwapImpactParams},
    PoolKind,
};

use crate::ExchangeError;

const SUPPORTED_POOLS: [PoolKind; 3] = [
    PoolKind::Primary,
    PoolKind::PriceImpact,
    PoolKind::ClaimableFee,
];

/// Accounts that can be used as [`Market`](gmx_core::Market).
pub trait AsMarket<'info>: Authentication<'info> {
    /// Get market.
    fn market(&self) -> AccountInfo<'info>;

    /// Get market token mint.
    fn market_token(&self) -> &Account<'info, Mint>;

    /// Get market sign of data store.
    fn market_sign(&self) -> AccountInfo<'info>;

    /// Get receiver.
    fn receiver(&self) -> &Account<'info, TokenAccount>;

    /// Get token program.
    fn token_program(&self) -> AccountInfo<'info>;

    /// Convert to a [`Market`](gmx_core::Market).
    fn as_market(&self) -> AccountsMarket<Self> {
        let mut pools = AccountsPools::new(self);
        for kind in SUPPORTED_POOLS {
            pools.add(kind);
        }
        AccountsMarket {
            accounts: self,
            pools,
        }
    }
}

/// Accounts pools.
struct AccountsPools<'a, T> {
    accounts: &'a T,
    pools: BTreeMap<PoolKind, AccountsPool<'a, T>>,
}

impl<'a, T> AccountsPools<'a, T> {
    fn new(accounts: &'a T) -> Self {
        Self {
            accounts,
            pools: BTreeMap::default(),
        }
    }

    fn add(&mut self, kind: PoolKind) -> &mut Self {
        self.pools.insert(
            kind,
            AccountsPool {
                kind,
                accounts: self.accounts,
            },
        );
        self
    }
}

/// Use [`Accounts`] as pool.
pub struct AccountsPool<'a, T> {
    kind: PoolKind,
    accounts: &'a T,
}

impl<'a, 'info, T> AccountsPool<'a, T>
where
    T: AsMarket<'info>,
{
    fn get_pool_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetPool<'info>> {
        let check_role = self.accounts.check_role_ctx();
        CpiContext::new(
            check_role.program,
            GetPool {
                market: self.accounts.market(),
            },
        )
    }

    fn pool(&self) -> Result<Option<Pool>> {
        Ok(data_store::cpi::get_pool(self.get_pool_ctx(), self.kind as u8)?.get())
    }

    fn apply_delta_to_market_pool_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, ApplyDeltaToMarketPool<'info>> {
        let check_role = self.accounts.check_role_ctx();
        CpiContext::new(
            check_role.program,
            ApplyDeltaToMarketPool {
                authority: self.accounts.authority().to_account_info(),
                store: check_role.accounts.store,
                only_controller: check_role.accounts.roles,
                market: self.accounts.market().to_account_info(),
            },
        )
    }
}

impl<'a, 'info, T> gmx_core::Pool for AccountsPool<'a, T>
where
    T: AsMarket<'info>,
{
    type Num = u128;

    type Signed = i128;

    fn long_token_amount(&self) -> gmx_core::Result<Self::Num> {
        let Some(pool) = self.pool()? else {
            return Ok(0);
        };
        Ok(pool.long_token_amount())
    }

    fn short_token_amount(&self) -> gmx_core::Result<Self::Num> {
        let Some(pool) = self.pool()? else {
            return Ok(0);
        };
        Ok(pool.short_token_amount())
    }

    fn apply_delta_to_long_token_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        data_store::cpi::apply_delta_to_market_pool(
            self.apply_delta_to_market_pool_ctx(),
            self.kind as u8,
            true,
            *delta,
        )?;
        Ok(())
    }

    fn apply_delta_to_short_token_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        data_store::cpi::apply_delta_to_market_pool(
            self.apply_delta_to_market_pool_ctx(),
            self.kind as u8,
            false,
            *delta,
        )?;
        Ok(())
    }
}

/// Use [`Accounts`] as market.
pub struct AccountsMarket<'a, T> {
    accounts: &'a T,
    pools: AccountsPools<'a, T>,
}

impl<'a, 'info, T> AccountsMarket<'a, T>
where
    T: AsMarket<'info>,
{
    fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintMarketTokenTo<'info>> {
        let check_role = self.accounts.check_role_ctx();
        CpiContext::new(
            check_role.program,
            MintMarketTokenTo {
                authority: self.accounts.authority().to_account_info(),
                only_controller: check_role.accounts.roles,
                store: check_role.accounts.store,
                market_token_mint: self.accounts.market_token().to_account_info(),
                to: self.accounts.receiver().to_account_info(),
                market_sign: self.accounts.market_sign(),
                token_program: self.accounts.token_program(),
            },
        )
    }
}

impl<'a, 'info, T> gmx_core::Market<{ constants::MARKET_DECIMALS }> for AccountsMarket<'a, T>
where
    T: AsMarket<'info>,
    'info: 'a,
{
    type Num = u128;

    type Signed = i128;

    type Pool = AccountsPool<'a, T>;

    fn pool(&self, kind: PoolKind) -> gmx_core::Result<Option<&Self::Pool>> {
        Ok(self.pools.pools.get(&kind))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> gmx_core::Result<Option<&mut Self::Pool>> {
        Ok(self.pools.pools.get_mut(&kind))
    }

    fn total_supply(&self) -> Self::Num {
        self.accounts.market_token().supply.into()
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        constants::MARKET_USD_TO_AMOUNT_DIVISOR
    }

    fn mint(&mut self, amount: &Self::Num) -> gmx_core::Result<()> {
        msg!("minting {}", amount);
        data_store::cpi::mint_market_token_to(
            self.mint_to_ctx(),
            (*amount)
                .try_into()
                .map_err(|_| gmx_core::Error::Overflow)?,
        )?;
        Ok(())
    }

    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num> {
        SwapImpactParams::builder()
            .with_exponent(2 * constants::MARKET_USD_UNIT)
            .with_positive_factor(2_000_000_000_000)
            .with_negative_factor(4_000_000_000_000)
            .build()
            .unwrap()
    }

    fn swap_fee_params(&self) -> FeeParams<Self::Num> {
        FeeParams::builder()
            .with_fee_receiver_factor(37_000_000_000_000_000_000)
            .with_positive_impact_factor(50_000_000_000_000_000)
            .with_negative_impact_factor(70_000_000_000_000_000)
            .build()
    }
}

pub(crate) struct GmxCoreError(gmx_core::Error);

impl From<gmx_core::Error> for GmxCoreError {
    fn from(err: gmx_core::Error) -> Self {
        Self(err)
    }
}

impl From<GmxCoreError> for Error {
    fn from(err: GmxCoreError) -> Self {
        match err.0 {
            gmx_core::Error::EmptyDeposit => ExchangeError::EmptyDepositAmounts.into(),
            gmx_core::Error::Solana(err) => err,
            _ => ExchangeError::FailedToExecuteDeposit.into(),
        }
    }
}
