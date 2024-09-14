use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    ops::{
        execution_fee::TransferExecutionFeeOps,
        withdrawal::{CreateWithdrawalOps, CreateWithdrawalParams},
    },
    states::{withdrawal::WithdrawalV2, Market, NonceBytes, Store},
    CoreError,
};

/// The accounts definitions for the `prepare_withdrawal_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareWithdrawalEscrow<'info> {
    /// The owner of the deposit.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The withdrawal owning these escrow accounts.
    /// CHECK: The withdrawal don't have to be initialized.
    #[account(
        seeds = [WithdrawalV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub withdrawal: UncheckedAccount<'info>,
    /// Market token.
    pub market_token: Box<Account<'info, Mint>>,
    /// Final long token.
    pub final_long_token: Box<Account<'info, Mint>>,
    /// Final short token.
    pub final_short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving market tokens to burn.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = market_token,
        associated_token::authority = withdrawal,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final long token
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = final_long_token,
        associated_token::authority = withdrawal,
    )]
    pub final_long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final short token
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = final_short_token,
        associated_token::authority = withdrawal,
    )]
    pub final_short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_withdrawal_escrow(
    _ctx: Context<PrepareWithdrawalEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definition for the `create_withdrawal` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateWithdrawal<'info> {
    /// The owner.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The withdrawal to be created.
    #[account(
        init,
        space = 8 + WithdrawalV2::INIT_SPACE,
        payer = owner,
        seeds = [WithdrawalV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub withdrawal: AccountLoader<'info, WithdrawalV2>,
    /// Market token.
    #[account(constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub market_token: Box<Account<'info, Mint>>,
    /// Final long token.
    pub final_long_token: Box<Account<'info, Mint>>,
    /// Final short token.
    pub final_short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving market tokens to burn.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = withdrawal,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final long tokens.
    #[account(
        mut,
        associated_token::mint = final_long_token,
        associated_token::authority = withdrawal,
    )]
    pub final_long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final short tokens.
    #[account(
        mut,
        associated_token::mint = final_short_token,
        associated_token::authority = withdrawal,
    )]
    pub final_short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The ATA of the owner for receving market tokens.
    #[account(
        mut,
        token::mint = market_token,
    )]
    pub market_token_source: Box<Account<'info, TokenAccount>>,
    /// The source final long token account.
    #[account(
        mut,
        associated_token::mint = final_long_token,
        associated_token::authority = owner,
    )]
    pub final_long_token_source: Box<Account<'info, TokenAccount>>,
    /// The source final short token account.
    #[account(
        mut,
        associated_token::mint = final_short_token,
        associated_token::authority = owner,
    )]
    pub final_short_token_source: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn create_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateWithdrawal<'info>>,
    nonce: NonceBytes,
    params: &CreateWithdrawalParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_fee(params)?;
    accounts.transfer_tokens(params)?;
    CreateWithdrawalOps::builder()
        .withdrawal(accounts.withdrawal.clone())
        .market(accounts.market.clone())
        .store(accounts.store.clone())
        .owner(&accounts.owner)
        .nonce(&nonce)
        .bump(ctx.bumps.withdrawal)
        .final_long_token(&accounts.final_long_token_escrow)
        .final_short_token(&accounts.final_short_token_escrow)
        .market_token(&accounts.market_token_escrow)
        .params(params)
        .swap_paths(ctx.remaining_accounts)
        .build()
        .execute()?;
    Ok(())
}

impl<'info> CreateWithdrawal<'info> {
    fn transfer_execution_fee(&self, params: &CreateWithdrawalParams) -> Result<()> {
        TransferExecutionFeeOps::builder()
            .payment(self.withdrawal.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_fee)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }

    fn transfer_tokens(&mut self, params: &CreateWithdrawalParams) -> Result<()> {
        let amount = params.market_token_amount;
        let source = &self.market_token_source;
        let target = &self.market_token_escrow;
        let mint = &self.market_token;
        if amount != 0 {
            transfer_checked(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferChecked {
                        from: source.to_account_info(),
                        mint: mint.to_account_info(),
                        to: target.to_account_info(),
                        authority: self.owner.to_account_info(),
                    },
                ),
                amount,
                mint.decimals,
            )?;
        }
        todo!()
    }
}
