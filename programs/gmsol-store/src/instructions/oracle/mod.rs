/// Instructions with price feeds.
pub mod price_feeds;

use anchor_lang::prelude::*;
use gmsol_utils::price::Price;

use crate::{
    states::{Oracle, Store},
    utils::internal,
};

pub use self::price_feeds::*;

/// The accounts definition for [`initialize_oracle`](crate::gmsol_store::initialize_oracle).
///
/// [*See also the documentation for the instruction.*](crate::gmsol_store::initialize_oracle)
#[derive(Accounts)]
#[instruction(index: u8)]
pub struct InitializeOracle<'info> {
    /// The authority of the instruction.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The store account that will be the owner of the oracle account.
    pub store: AccountLoader<'info, Store>,
    /// The new oracle account.
    #[account(
        init,
        payer = authority,
        space = 8 + Oracle::INIT_SPACE,
        seeds = [Oracle::SEED, store.key().as_ref(), &[index]],
        bump,
    )]
    pub oracle: Account<'info, Oracle>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// Initialize an [`Oracle`] account with the given `index`.
///
/// ## CHECK
/// - Only MARKET_KEEPER can perform this action.
pub(crate) fn unchecked_initialize_oracle(ctx: Context<InitializeOracle>, index: u8) -> Result<()> {
    ctx.accounts
        .oracle
        .init(ctx.bumps.oracle, ctx.accounts.store.key(), index);
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeOracle<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct ClearAllPrices<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Oracle::SEED, store.key().as_ref(), &[oracle.index]],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
}

/// Clear all prices of the given oracle account.
pub(crate) fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
    ctx.accounts.oracle.clear_all_prices();
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

#[derive(Accounts)]
pub struct SetPrice<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Oracle::SEED, store.key().as_ref(), &[oracle.index]],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
}

impl<'info> internal::Authentication<'info> for SetPrice<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// Set the price of a token in the given oracle.
/// # Error
/// Returns error if the price of the given token already been set.
pub(crate) fn set_price(ctx: Context<SetPrice>, token: Pubkey, price: Price) -> Result<()> {
    ctx.accounts.oracle.primary.set(&token, price)
}
