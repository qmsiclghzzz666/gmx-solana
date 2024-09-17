use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::states::{order::OrderV2, NonceBytes, Store};

/// The accounts definitions for the `prepare_swap_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareSwapOrderEscrow<'info> {
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
    /// Swap in token (will be stored as initial collateral token in order account).
    pub swap_in_token: Box<Account<'info, Mint>>,
    /// Swap out token (will be stored as collateral/output token in order account).
    pub swap_out_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving the swap in tokens from the owner.
    /// It will be stored as initial collateral token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = swap_in_token,
        associated_token::authority = order,
    )]
    pub swap_in_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the swap out tokens after the swap.
    /// It will be stored as final output token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = swap_out_token,
        associated_token::authority = order,
    )]
    pub swap_out_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_swap_order_escrow(
    _ctx: Context<PrepareSwapOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definitions for the `prepare_increase_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareIncreaseOrderEscrow<'info> {
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
    /// Initial collateral token (will be stored as initial collateral token in order account).
    pub initial_collateral_token: Box<Account<'info, Mint>>,
    /// Long token of the market.
    pub long_token: Box<Account<'info, Mint>>,
    /// Short token of the market.
    pub short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving the initial collateral tokens from the owner.
    /// It will be stored as initial collateral token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = initial_collateral_token,
        associated_token::authority = order,
    )]
    pub initial_collateral_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in long tokens after increasing position.
    /// It will be stored as long token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in short tokens after increasing position.
    /// It will be stored as short token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_increase_order_escrow(
    _ctx: Context<PrepareIncreaseOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definitions for the `prepare_decrease_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareDecreaseOrderEscrow<'info> {
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
    /// Final output token (will be stored as final output token in order account).
    pub final_output_token: Box<Account<'info, Mint>>,
    /// Long token of the market.
    pub long_token: Box<Account<'info, Mint>>,
    /// Short token of the market.
    pub short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving final output tokens after decreasing position.
    /// It will be stored as final output token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = final_output_token,
        associated_token::authority = order,
    )]
    pub initial_collateral_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in long tokens or pnl tokens after decreasing position.
    /// It will be stored as long token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in short tokens or pnl tokens after decreasing position.
    /// It will be stored as short token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_decrease_order_escrow(
    _ctx: Context<PrepareDecreaseOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}
