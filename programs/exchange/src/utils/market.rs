use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use data_store::{
    cpi::accounts::{ApplyDeltaToMarketPool, MintMarketTokenTo},
    states::{Market, Pool, PoolKind},
    utils::Authentication,
};

use crate::ExchangeError;

/// Accounts that can be used as [`Market`](gmx_core::Market).
pub trait AsMarket<'info>: Authentication<'info> {
    /// Get market.
    fn market(&self) -> &Account<'info, Market>;

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
        AccountsMarket {
            accounts: self,
            primary: AccountsPool {
                kind: PoolKind::Primary,
                accounts: self,
            },
            price_impact: AccountsPool {
                kind: PoolKind::PriceImpact,
                accounts: self,
            },
        }
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
    fn pool(&'a self) -> &'a Pool
    where
        'info: 'a,
    {
        match self.kind {
            PoolKind::Primary => &self.accounts.market().primary,
            PoolKind::PriceImpact => &self.accounts.market().price_impact,
        }
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

    fn long_token_amount(&self) -> Self::Num {
        self.pool().long_token_amount()
    }

    fn short_token_amount(&self) -> Self::Num {
        self.pool().short_token_amount()
    }

    fn apply_delta_to_long_token_amount(&mut self, delta: Self::Signed) -> gmx_core::Result<()> {
        data_store::cpi::apply_delta_to_market_pool(
            self.apply_delta_to_market_pool_ctx(),
            self.kind as u8,
            true,
            delta,
        )?;
        Ok(())
    }

    fn apply_delta_to_short_token_amount(&mut self, delta: Self::Signed) -> gmx_core::Result<()> {
        data_store::cpi::apply_delta_to_market_pool(
            self.apply_delta_to_market_pool_ctx(),
            self.kind as u8,
            false,
            delta,
        )?;
        Ok(())
    }
}

/// Use [`Accounts`] as market.
pub struct AccountsMarket<'a, T> {
    accounts: &'a T,
    primary: AccountsPool<'a, T>,
    price_impact: AccountsPool<'a, T>,
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

impl<'a, 'info, T> gmx_core::Market for AccountsMarket<'a, T>
where
    T: AsMarket<'info>,
    'info: 'a,
{
    type Num = u128;

    type Signed = i128;

    type Pool = AccountsPool<'a, T>;

    fn pool(&self) -> &Self::Pool {
        &self.primary
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.primary
    }

    fn price_impact_pool(&self) -> &Self::Pool {
        &self.price_impact
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.price_impact
    }

    fn total_supply(&self) -> Self::Num {
        self.accounts.market_token().supply.into()
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        Market::USD_TO_AMOUNT_DIVISOR
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
