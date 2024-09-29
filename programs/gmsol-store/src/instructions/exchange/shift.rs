use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    ops::{
        execution_fee::TransferExecutionFeeOps,
        shift::{CreateShiftOp, CreateShiftParams},
    },
    states::{Market, NonceBytes, Seed, Shift, Store},
    CoreError,
};

/// The accounts definition for the `create_shift` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateShift<'info> {
    /// The owner.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// From market.
    #[account(mut, has_one = store)]
    pub from_market: AccountLoader<'info, Market>,
    /// To market.
    #[account(
        has_one = store,
        constraint = from_market.load()?.validate_shiftable(&*to_market.load()?).is_ok() @ CoreError::TokenMintMismatched,
    )]
    pub to_market: AccountLoader<'info, Market>,
    /// Shift.
    #[account(
        init,
        space = 8 + Shift::INIT_SPACE,
        payer = owner,
        seeds = [Shift::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub shift: AccountLoader<'info, Shift>,
    /// From market token.
    #[account(constraint = from_market.load()?.meta().market_token_mint == from_market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub from_market_token: Box<Account<'info, Mint>>,
    /// To market token.
    #[account(constraint = from_market.load()?.meta().market_token_mint == from_market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub to_market_token: Box<Account<'info, Mint>>,
    /// The escrow account for the from market tokens.
    #[account(
        mut,
        associated_token::mint = from_market_token,
        associated_token::authority = shift,
    )]
    pub from_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for the to market tokens.
    #[account(
        mut,
        associated_token::mint = to_market_token,
        associated_token::authority = shift,
    )]
    pub to_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The source from market token account.
    #[account(
        mut,
        token::mint = from_market_token,
    )]
    pub from_market_token_source: Box<Account<'info, TokenAccount>>,
    /// The ATA for receiving to market tokens.
    #[account(
        associated_token::mint = to_market_token,
        associated_token::authority = owner,
    )]
    pub to_market_tokne_ata: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn create_shift(
    ctx: Context<CreateShift>,
    nonce: &NonceBytes,
    params: &CreateShiftParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_fee(params)?;
    accounts.transfer_tokens(params)?;
    CreateShiftOp::builder()
        .store(&accounts.store)
        .owner(accounts.owner.to_account_info())
        .shift(&accounts.shift)
        .from_market(&accounts.from_market)
        .from_market_token_account(&accounts.from_market_token_escrow)
        .to_market(&accounts.to_market)
        .to_market_token_account(&accounts.to_market_token_escrow)
        .nonce(nonce)
        .bump(ctx.bumps.shift)
        .params(params)
        .build()
        .execute()?;
    Ok(())
}

impl<'info> CreateShift<'info> {
    fn transfer_execution_fee(&self, params: &CreateShiftParams) -> Result<()> {
        TransferExecutionFeeOps::builder()
            .payment(self.shift.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_lamports)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }

    fn transfer_tokens(&mut self, params: &CreateShiftParams) -> Result<()> {
        let amount = params.from_market_token_amount;
        let source = &self.from_market_token_source;
        let target = &mut self.from_market_token_escrow;
        let mint = &self.from_market_token;
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
            target.reload()?;
        }
        Ok(())
    }
}
