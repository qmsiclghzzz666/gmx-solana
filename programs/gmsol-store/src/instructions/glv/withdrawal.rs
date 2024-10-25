use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_2022::Token2022,
    token_interface,
};
use gmsol_utils::InitSpace;

use crate::{
    ops::{
        execution_fee::TransferExecutionFeeOperation,
        glv::{CreateGlvWithdrawalOperation, CreateGlvWithdrawalParams},
    },
    states::{glv::GlvWithdrawal, Glv, Market, NonceBytes, Seed, Store},
    CoreError,
};

/// The accounts defintion for [`create_glv_withdrawal`] instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateGlvWithdrawal<'info> {
    /// Owner.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Market.
    #[account(
        mut,
        has_one = store,
        constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched,
    )]
    pub market: AccountLoader<'info, Market>,
    /// GLV.
    #[account(
        has_one = store,
        constraint = glv.load()?.glv_token == glv_token.key() @ CoreError::TokenMintMismatched,
        constraint = glv.load()?.contains(&market_token.key()) @ CoreError::InvalidArgument,
    )]
    pub glv: AccountLoader<'info, Glv>,
    /// GLV withdrawal.
    #[account(
        init,
        payer = owner,
        space = 8 + GlvWithdrawal::INIT_SPACE,
        seeds = [GlvWithdrawal::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub glv_withdrawal: AccountLoader<'info, GlvWithdrawal>,
    /// GLV Token.
    pub glv_token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    /// Market token.
    pub market_token: Box<Account<'info, Mint>>,
    /// Final long token.
    pub final_long_token: Box<Account<'info, Mint>>,
    /// Final short token.
    pub final_short_token: Box<Account<'info, Mint>>,
    /// The source GLV token account.
    #[account(mut, token::mint = glv_token)]
    pub glv_token_source: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    /// The escrow account for GLV tokens.
    #[account(
        mut,
        associated_token::mint = glv_token,
        associated_token::authority = glv_withdrawal,
        associated_token::token_program = glv_token_program,
    )]
    pub glv_token_escrow: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    /// The escrow account for market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = glv_withdrawal,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for long tokens.
    #[account(
        mut,
        associated_token::mint = final_long_token,
        associated_token::authority = glv_withdrawal,
    )]
    pub final_long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for short tokens.
    #[account(
        mut,
        associated_token::mint = final_short_token,
        associated_token::authority = glv_withdrawal,
    )]
    pub final_short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The token program for GLV token.
    pub glv_token_program: Program<'info, Token2022>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn create_glv_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateGlvWithdrawal<'info>>,
    nonce: &NonceBytes,
    params: &CreateGlvWithdrawalParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_lamports(params)?;
    accounts.transfer_glv_tokens(params)?;
    CreateGlvWithdrawalOperation::builder()
        .glv_withdrawal(accounts.glv_withdrawal.clone())
        .market(accounts.market.clone())
        .store(accounts.store.clone())
        .owner(&accounts.owner)
        .nonce(nonce)
        .bump(ctx.bumps.glv_withdrawal)
        .final_long_token(&accounts.final_long_token_escrow)
        .final_short_token(&accounts.final_short_token_escrow)
        .market_token(&accounts.market_token_escrow)
        .glv_token(&accounts.glv_token_escrow)
        .params(params)
        .swap_paths(ctx.remaining_accounts)
        .build()
        .unchecked_execute()?;
    Ok(())
}

impl<'info> CreateGlvWithdrawal<'info> {
    fn transfer_execution_lamports(&self, params: &CreateGlvWithdrawalParams) -> Result<()> {
        TransferExecutionFeeOperation::builder()
            .payment(self.glv_withdrawal.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_lamports)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }

    fn transfer_glv_tokens(&mut self, params: &CreateGlvWithdrawalParams) -> Result<()> {
        use anchor_spl::token_interface::{transfer_checked, TransferChecked};

        let amount = params.glv_token_amount;
        require!(amount != 0, CoreError::EmptyGlvWithdrawal);

        let source = &self.glv_token_source;
        let target = &self.glv_token_escrow;
        let mint = &self.glv_token;

        transfer_checked(
            CpiContext::new(
                self.glv_token_program.to_account_info(),
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

        Ok(())
    }
}
