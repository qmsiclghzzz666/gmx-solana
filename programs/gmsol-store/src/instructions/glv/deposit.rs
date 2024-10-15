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
        glv::{CreateGlvDepositOperation, CreateGlvDepositParams},
    },
    states::{Glv, GlvDeposit, Market, NonceBytes, Seed, Store},
    CoreError,
};

/// The accounts definitions for [`create_glv_deposit`] instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateGlvDeposit<'info> {
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
        constraint = glv.load()?.market_tokens().contains(&market_token.key()) @ CoreError::InvalidArgument,
    )]
    pub glv: AccountLoader<'info, Glv>,
    /// GLV deposit.
    #[account(
        init,
        payer = owner,
        space = 8 + GlvDeposit::INIT_SPACE,
        seeds = [GlvDeposit::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub glv_deposit: AccountLoader<'info, GlvDeposit>,
    /// GLV Token.
    pub glv_token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    /// Market token.
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// initial short token.
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The source market token account.
    #[account(mut, token::mint = market_token)]
    pub market_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The source initial long token account.
    #[account(mut, token::mint = initial_long_token)]
    pub initial_long_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The source initial short token account.
    #[account(mut, token::mint = initial_short_token)]
    pub initial_short_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for glv tokens.
    #[account(
        mut,
        associated_token::mint = glv_token,
        associated_token::authority = glv_deposit,
        associated_token::token_program = glv_token_program,
    )]
    pub glv_token_escrow: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    /// The escrow account for market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = glv_deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for initial long tokens.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for initial short tokens.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The token program for GLV token.
    pub glv_token_program: Program<'info, Token2022>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn create_glv_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateGlvDeposit<'info>>,
    nonce: &NonceBytes,
    params: &CreateGlvDepositParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_lamports(params)?;
    accounts.transfer_tokens(params)?;
    CreateGlvDepositOperation::builder()
        .glv_deposit(accounts.glv_deposit.clone())
        .market(accounts.market.clone())
        .store(accounts.store.clone())
        .owner(accounts.owner.to_account_info())
        .nonce(nonce)
        .bump(ctx.bumps.glv_deposit)
        .initial_long_token(accounts.initial_long_token_escrow.as_deref())
        .initial_short_token(accounts.initial_short_token_escrow.as_deref())
        .market_token(&accounts.market_token_escrow)
        .glv_token(&accounts.glv_token_escrow)
        .params(params)
        .swap_paths(ctx.remaining_accounts)
        .build()
        .unchecked_execute()?;
    Ok(())
}

impl<'info> CreateGlvDeposit<'info> {
    fn transfer_execution_lamports(&self, params: &CreateGlvDepositParams) -> Result<()> {
        TransferExecutionFeeOperation::builder()
            .payment(self.glv_deposit.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_lamports)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }

    fn transfer_tokens(&mut self, params: &CreateGlvDepositParams) -> Result<()> {
        use anchor_spl::token::{transfer_checked, TransferChecked};

        let amount = params.initial_long_token_amount;
        if amount != 0 {
            let Some(source) = self.initial_long_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_long_token_escrow.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_long_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
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

        let amount = params.initial_short_token_amount;
        if amount != 0 {
            let Some(source) = self.initial_short_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_short_token_escrow.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_short_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
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

        let amount = params.market_token_amount;
        if amount != 0 {
            let Some(source) = self.market_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let target = &self.market_token_escrow;
            let mint = &self.market_token;
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

        // Make sure the data for escrow accounts is up-to-date.
        for escrow in self
            .initial_long_token_escrow
            .as_mut()
            .into_iter()
            .chain(self.initial_short_token_escrow.as_mut())
            .chain(Some(&mut self.market_token_escrow))
        {
            escrow.reload()?;
        }

        Ok(())
    }
}
