use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::TokenAccount;

use crate::{
    states::{
        common::SwapParams,
        deposit::{Receivers, TokenParams},
        DataStore, Deposit, Market, NonceBytes, Roles, Seed,
    },
    utils::internal,
    DataStoreError,
};

/// Initialize a new [`Deposit`] account.
pub fn initialize_deposit(
    ctx: Context<InitializeDeposit>,
    nonce: NonceBytes,
    tokens_with_feed: Vec<(Pubkey, Pubkey)>,
    swap_params: SwapParams,
    token_params: TokenParams,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    ctx.accounts.deposit.init(
        ctx.bumps.deposit,
        &ctx.accounts.market,
        nonce,
        tokens_with_feed,
        ctx.accounts.payer.key(),
        ctx.accounts.initial_long_token_account.as_ref(),
        ctx.accounts.initial_short_token_account.as_ref(),
        Receivers {
            receiver: ctx.accounts.receiver.key(),
            ui_fee_receiver,
        },
        token_params,
        swap_params,
    )
}

#[derive(Accounts)]
#[instruction(nonce: [u8; 32], tokens_with_feed: Vec<(Pubkey, Pubkey)>, swap_params: SwapParams)]
pub struct InitializeDeposit<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        space = 8 + Deposit::init_space(&tokens_with_feed, &swap_params),
        payer = payer,
        seeds = [Deposit::SEED, store.key().as_ref(), payer.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: Account<'info, Deposit>,
    #[account(token::authority = payer)]
    pub initial_long_token_account: Option<Account<'info, TokenAccount>>,
    #[account(token::authority = payer)]
    pub initial_short_token_account: Option<Account<'info, TokenAccount>>,
    pub(crate) market: Account<'info, Market>,
    #[account(token::authority = payer, token::mint = market.meta.market_token_mint)]
    pub receiver: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for InitializeDeposit<'info> {
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

/// Remove a deposit.
pub fn remove_deposit(ctx: Context<RemoveDeposit>, refund: u64) -> Result<()> {
    system_program::transfer(ctx.accounts.transfer_ctx(), refund)
}

#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        constraint = deposit.to_account_info().lamports() >= refund @ DataStoreError::LamportsNotEnough,
        close = authority,
        constraint = deposit.fixed.senders.user == user.key() @ DataStoreError::UserMismatch,
        seeds = [
            Deposit::SEED,
            store.key().as_ref(),
            user.key().as_ref(),
            &deposit.fixed.nonce,
        ],
        bump = deposit.fixed.bump,
    )]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports,
    /// and has been checked in `deposit`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for RemoveDeposit<'info> {
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

impl<'info> RemoveDeposit<'info> {
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
