use anchor_lang::{prelude::*, system_program};

use crate::{
    states::{Position, Seed, Store},
    utils::internal,
    DataStoreError,
};

#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemovePosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        constraint = position.to_account_info().lamports() >= refund @ DataStoreError::LamportsNotEnough,
        close = payer,
        constraint = position.load()?.owner == user.key() @ DataStoreError::UserMismatch,
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            position.load()?.owner.as_ref(),
            position.load()?.market_token.as_ref(),
            position.load()?.collateral_token.as_ref(),
            &[position.load()?.kind],
        ],
        bump = position.load()?.bump,
    )]
    pub position: AccountLoader<'info, Position>,
    /// CHECK: only used to receive lamports,
    /// and has been checked in `position`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

/// Remove a position.
pub fn remove_position(ctx: Context<RemovePosition>, refund: u64) -> Result<()> {
    system_program::transfer(ctx.accounts.transfer_ctx(), refund)
}

impl<'info> internal::Authentication<'info> for RemovePosition<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> RemovePosition<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.user.to_account_info(),
            },
        )
    }
}
