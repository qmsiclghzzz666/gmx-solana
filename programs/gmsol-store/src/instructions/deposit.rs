use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    events::RemoveDepositEvent,
    states::{
        common::{SwapParams, TokenRecord},
        deposit::{Receivers, TokenParams},
        Deposit, Market, NonceBytes, Seed, Store,
    },
    utils::internal,
    StoreError,
};

#[derive(Accounts)]
#[instruction(nonce: [u8; 32], tokens_with_feed: Vec<TokenRecord>, swap_params: SwapParams)]
pub struct InitializeDeposit<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        init,
        space = 8 + Deposit::init_space(&tokens_with_feed, &swap_params),
        payer = payer,
        seeds = [Deposit::SEED, store.key().as_ref(), payer.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: Box<Account<'info, Deposit>>,
    #[account(token::authority = payer)]
    pub initial_long_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(token::authority = payer)]
    pub initial_short_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, has_one = store)]
    pub(crate) market: AccountLoader<'info, Market>,
    #[account(token::authority = payer, token::mint = market.load()?.meta().market_token_mint)]
    pub receiver: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Initialize a new [`Deposit`] account.
pub fn initialize_deposit(
    ctx: Context<InitializeDeposit>,
    nonce: NonceBytes,
    tokens_with_feed: Vec<TokenRecord>,
    swap_params: SwapParams,
    token_params: TokenParams,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    require!(
        token_params.initial_long_token_amount != 0 || token_params.initial_short_token_amount != 0,
        StoreError::EmptyDeposit
    );
    let id = ctx
        .accounts
        .market
        .load_mut()?
        .state_mut()
        .next_deposit_id()?;
    ctx.accounts.deposit.init(
        ctx.bumps.deposit,
        id,
        ctx.accounts.store.key(),
        &ctx.accounts.market,
        nonce,
        tokens_with_feed,
        ctx.accounts.payer.key(),
        ctx.accounts.initial_long_token_account.as_deref(),
        ctx.accounts.initial_short_token_account.as_deref(),
        Receivers {
            receiver: ctx.accounts.receiver.key(),
            ui_fee_receiver,
        },
        token_params,
        swap_params,
    )
}

impl<'info> internal::Authentication<'info> for InitializeDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveDeposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        close = payer,
        constraint = deposit.to_account_info().lamports() >= refund @ StoreError::LamportsNotEnough,
        constraint = deposit.fixed.store == store.key() @ StoreError::InvalidDepositToRemove,
        constraint = deposit.fixed.senders.user == user.key() @ StoreError::UserMismatch,
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Remove a deposit.
pub fn remove_deposit(ctx: Context<RemoveDeposit>, refund: u64, reason: &str) -> Result<()> {
    system_program::transfer(ctx.accounts.transfer_ctx(), refund)?;
    emit_cpi!(RemoveDepositEvent::new(
        ctx.accounts.store.key(),
        ctx.accounts.deposit.key(),
        ctx.accounts.deposit.fixed.tokens.market_token,
        ctx.accounts.deposit.fixed.senders.user,
        reason,
    )?);
    Ok(())
}

impl<'info> internal::Authentication<'info> for RemoveDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> RemoveDeposit<'info> {
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
