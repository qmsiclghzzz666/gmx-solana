use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::TokenAccount;

use crate::{
    states::{
        common::SwapParams, withdrawal::TokenParams, DataStore, Market, NonceBytes, Roles, Seed,
        Withdrawal,
    },
    utils::internal,
    DataStoreError,
};

/// Initialize a new [`Withdrawal`] account.
pub fn initialize_withdrawal(
    ctx: Context<InitializeWithdrawal>,
    nonce: NonceBytes,
    swap_params: SwapParams,
    tokens_with_feed: Vec<(Pubkey, Pubkey)>,
    tokens_params: TokenParams,
    market_token_amount: u64,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    ctx.accounts.withdrawal.init(
        ctx.bumps.withdrawal,
        nonce,
        ctx.accounts.payer.key(),
        &ctx.accounts.market,
        ctx.accounts.market_token_account.key(),
        market_token_amount,
        tokens_params,
        swap_params,
        tokens_with_feed,
        &ctx.accounts.final_long_token_receiver,
        &ctx.accounts.final_short_token_receiver,
        ui_fee_receiver,
    )
}

#[derive(Accounts)]
#[instruction(nonce: [u8; 32], swap_params: SwapParams, tokens_with_feed: Vec<(Pubkey, Pubkey)>)]
pub struct InitializeWithdrawal<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        space = 8 + Withdrawal::init_space(&tokens_with_feed, &swap_params),
        payer = payer,
        seeds = [Withdrawal::SEED, store.key().as_ref(), payer.key().as_ref(), &nonce],
        bump,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    pub(crate) market: Account<'info, Market>,
    #[account(token::authority = payer, token::mint = market.meta.market_token_mint)]
    pub market_token_account: Account<'info, TokenAccount>,
    #[account(token::authority = payer)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(token::authority = payer)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for InitializeWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

/// Remove a withdrawal.
pub fn remove_withdrawal(ctx: Context<RemoveWithdrawal>, refund: u64) -> Result<()> {
    system_program::transfer(ctx.accounts.transfer_ctx(), refund)
}

#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveWithdrawal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(
        mut,
        constraint = withdrawal.to_account_info().lamports() >= refund @ DataStoreError::LamportsNotEnough,
        close = authority,
        constraint = withdrawal.fixed.user == user.key() @ DataStoreError::UserMismatch,
        seeds = [
            Withdrawal::SEED,
            store.key().as_ref(),
            user.key().as_ref(),
            &withdrawal.fixed.nonce,
        ],
        bump = withdrawal.fixed.bump,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    /// CHECK: only used to receive the refund, and has been checked
    /// to be the valid receiver in `withdrawal`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for RemoveWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

impl<'info> RemoveWithdrawal<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.authority.to_account_info(),
                to: self.user.to_account_info(),
            },
        )
    }
}
