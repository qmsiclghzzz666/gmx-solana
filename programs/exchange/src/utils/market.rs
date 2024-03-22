use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use data_store::{
    cpi::accounts::MintMarketTokenTo,
    states::{Market, Pool},
    utils::Authentication,
};

use crate::ExchangeError;

/// Accounts that can be used as [`Market`](gmx_core::Market).
pub trait AsMarket<'info>: Authentication<'info> {
    /// Get market.
    fn market(&self) -> &Account<'info, Market>;

    /// Get mutable market.
    fn market_mut(&mut self) -> &mut Account<'info, Market>;

    /// Get market token mint.
    fn market_token(&self) -> &Account<'info, Mint>;

    /// Get market sign of data store.
    fn market_sign(&self) -> AccountInfo<'info>;

    /// Get receiver.
    fn receiver(&self) -> &Account<'info, TokenAccount>;

    /// Get token program.
    fn token_program(&self) -> AccountInfo<'info>;

    /// Convert to a [`Market`](gmx_core::Market).
    fn as_market(&mut self) -> AccountsMarket<Self> {
        AccountsMarket { accounts: self }
    }
}

/// Use [`Accounts`] as market.
pub struct AccountsMarket<'a, T> {
    accounts: &'a mut T,
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

    type Pool = Pool;

    fn pool(&self) -> &Self::Pool {
        &self.accounts.market().primary
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.accounts.market_mut().primary
    }

    fn price_impact_pool(&self) -> &Self::Pool {
        &self.accounts.market().price_impact
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.accounts.market_mut().price_impact
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
