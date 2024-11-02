/// Instructions with price feeds.
pub mod price_feeds;

use anchor_lang::prelude::*;

use crate::{
    states::{Oracle, Store},
    utils::internal,
};

pub use self::price_feeds::*;

/// The accounts definition for [`initialize_oracle`](crate::gmsol_store::initialize_oracle).
///
/// [*See also the documentation for the instruction.*](crate::gmsol_store::initialize_oracle)
#[derive(Accounts)]
pub struct InitializeOracle<'info> {
    pub payer: Signer<'info>,
    /// The store account that will be the owner of the oracle account.
    pub store: AccountLoader<'info, Store>,
    /// The new oracle account.
    #[account(zero)]
    pub oracle: AccountLoader<'info, Oracle>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

pub(crate) fn unchecked_initialize_oracle(ctx: Context<InitializeOracle>) -> Result<()> {
    ctx.accounts
        .oracle
        .load_init()?
        .init(ctx.accounts.store.key());
    Ok(())
}

#[derive(Accounts)]
pub struct ClearAllPrices<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
    )]
    pub oracle: AccountLoader<'info, Oracle>,
}

/// Clear all prices of the given oracle account.
pub(crate) fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
    ctx.accounts.oracle.load_mut()?.clear_all_prices();
    Ok(())
}

impl<'info> internal::Authentication<'info> for ClearAllPrices<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
