use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    constants,
    states::{
        common::{SwapParams, TokenRecord},
        withdrawal::TokenParams,
        Market, NonceBytes, Seed, Store, Withdrawal,
    },
    utils::internal,
    DataStoreError,
};

#[derive(Accounts)]
#[instruction(nonce: [u8; 32], swap_params: SwapParams, tokens_with_feed: Vec<TokenRecord>)]
pub struct InitializeWithdrawal<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
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
    #[account(has_one = store)]
    pub(crate) market: Account<'info, Market>,
    #[account(mut, token::authority = payer, token::mint = market.meta.market_token_mint)]
    pub market_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = market_token_account.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_withdrawal_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    #[account(token::authority = payer)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(token::authority = payer)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Initialize a new [`Withdrawal`] account.
pub fn initialize_withdrawal(
    ctx: Context<InitializeWithdrawal>,
    nonce: NonceBytes,
    swap_params: SwapParams,
    tokens_with_feed: Vec<TokenRecord>,
    tokens_params: TokenParams,
    market_token_amount: u64,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    require!(market_token_amount != 0, DataStoreError::EmptyWithdrawal);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.market_token_account.to_account_info(),
                to: ctx.accounts.market_token_withdrawal_vault.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        market_token_amount,
    )?;
    ctx.accounts.withdrawal.init(
        ctx.bumps.withdrawal,
        ctx.accounts.store.key(),
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

impl<'info> internal::Authentication<'info> for InitializeWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveWithdrawal<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        close = payer,
        constraint = withdrawal.fixed.store == store.key() @ DataStoreError::InvalidWithdrawalToRemove,
        constraint = withdrawal.to_account_info().lamports() >= refund @ DataStoreError::LamportsNotEnough,
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
    /// Token account for receiving the market tokens.
    #[account(
        mut,
        token::authority = user,
        constraint = withdrawal.fixed.market_token_account == market_token.key() @ DataStoreError::InvalidWithdrawalToRemove,
    )]
    pub market_token: Option<Account<'info, TokenAccount>>,
    /// The vault saving the market tokens for withdrawal.
    #[account(
        mut,
        token::mint = market_token.as_ref().expect("must provided").mint,
        constraint = withdrawal.fixed.tokens.market_token == market_token.as_ref().expect("must provided").mint @ DataStoreError::InvalidWithdrawalToRemove,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_withdrawal_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub market_token_withdrawal_vault: Option<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Remove a withdrawal.
pub fn remove_withdrawal(ctx: Context<RemoveWithdrawal>, refund: u64) -> Result<()> {
    use crate::internal::TransferUtils;

    let amount = ctx.accounts.withdrawal.fixed.tokens.market_token_amount;

    if amount != 0 {
        TransferUtils::new(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.store,
            None,
        )
        .transfer_out(
            ctx.accounts
                .market_token_withdrawal_vault
                .as_ref()
                .ok_or(error!(
                    DataStoreError::UnableToTransferOutRemainingWithdrawalAmount
                ))?
                .to_account_info(),
            ctx.accounts
                .market_token
                .as_ref()
                .ok_or(error!(
                    DataStoreError::UnableToTransferOutRemainingWithdrawalAmount
                ))?
                .to_account_info(),
            amount,
        )?;
    }

    system_program::transfer(ctx.accounts.transfer_ctx(), refund)
}

impl<'info> internal::Authentication<'info> for RemoveWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> RemoveWithdrawal<'info> {
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
