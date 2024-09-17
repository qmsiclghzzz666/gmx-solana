use anchor_lang::prelude::*;

use crate::states::{order::OrderV2, Store};

/// The accounts definitions for the `prepare_swap_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareSwapEscrow<'info> {
    /// The owner of the order.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The order owning these escrow accounts.
    /// CHECK: The order account don't have to be initialized.
    #[account(
        seeds = [OrderV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub order: UncheckedAccount<'info>,
}
