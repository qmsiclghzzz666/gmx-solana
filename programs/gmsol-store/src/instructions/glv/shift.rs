use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use gmsol_utils::InitSpace;

use crate::{
    ops::shift::{CreateShiftOperation, CreateShiftParams},
    states::{common::action, glv::GlvShift, Glv, Market, NonceBytes, Seed, Store},
    utils::internal,
    CoreError,
};

/// The accounts definition for [`create_glv_shift`] instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateGlvShift<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// GLV.
    #[account(
        has_one = store,
        constraint = glv.load()?.contains(&from_market_token.key()) @ CoreError::InvalidArgument,
        constraint = glv.load()?.contains(&to_market_token.key()) @ CoreError::InvalidArgument,
    )]
    pub glv: AccountLoader<'info, Glv>,
    /// From market.
    #[account(
        mut,
        has_one = store,
        constraint = from_market.load()?.meta().market_token_mint == from_market_token.key() @ CoreError::MarketTokenMintMismatched,
    )]
    pub from_market: AccountLoader<'info, Market>,
    /// To market.
    #[account(
        mut,
        has_one = store,
        constraint = to_market.load()?.meta().market_token_mint == to_market_token.key() @ CoreError::MarketTokenMintMismatched,
    )]
    pub to_market: AccountLoader<'info, Market>,
    /// GLV shift.
    #[account(
        init,
        payer = authority,
        space = 8 + GlvShift::INIT_SPACE,
        seeds = [GlvShift::SEED, store.key().as_ref(), glv.key().as_ref(), &nonce],
        bump,
    )]
    pub glv_shift: AccountLoader<'info, GlvShift>,
    /// From market token.
    #[account(
        constraint = from_market_token.key() != to_market_token.key() @ CoreError::InvalidShiftMarkets,
    )]
    pub from_market_token: Box<Account<'info, Mint>>,
    /// To market token.
    pub to_market_token: Box<Account<'info, Mint>>,
    /// Vault for from market tokens.
    #[account(
        mut,
        associated_token::mint = from_market_token,
        associated_token::authority = glv,
    )]
    pub from_market_token_vault: Box<Account<'info, TokenAccount>>,
    /// The escrow account for from market tokens.
    #[account(
        mut,
        associated_token::mint = from_market_token,
        associated_token::authority = glv_shift,
    )]
    pub from_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for to market tokens.
    #[account(
        associated_token::mint = to_market_token,
        associated_token::authority = glv_shift,
    )]
    pub to_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> CreateGlvShift<'info> {
    fn transfer_from_market_tokens(&mut self, amount: u64) -> Result<()> {
        use anchor_spl::token_interface::{transfer_checked, TransferChecked};

        require!(amount != 0, CoreError::EmptyShift);

        let source = &self.from_market_token_vault;
        let target = &self.from_market_token_escrow;
        let mint = &self.from_market_token;

        let glv = self.glv.load()?;

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: source.to_account_info(),
                    mint: mint.to_account_info(),
                    to: target.to_account_info(),
                    authority: self.glv.to_account_info(),
                },
            )
            .with_signer(&[&glv.signer_seeds()]),
            amount,
            mint.decimals,
        )?;

        Ok(())
    }
}

impl<'info> action::Create<'info, GlvShift> for CreateGlvShift<'info> {
    type CreateParams = CreateShiftParams;

    fn action(&self) -> AccountInfo<'info> {
        self.glv_shift.to_account_info()
    }

    fn payer(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn system_program(&self) -> AccountInfo<'info> {
        self.system_program.to_account_info()
    }

    fn create_impl(
        &mut self,
        params: &Self::CreateParams,
        nonce: &NonceBytes,
        bumps: &Self::Bumps,
        _remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        self.transfer_from_market_tokens(params.from_market_token_amount)?;
        CreateShiftOperation::builder()
            .store(&self.store)
            .owner(self.glv.to_account_info())
            .shift(&self.glv_shift)
            .from_market(&self.from_market)
            .from_market_token_account(&self.from_market_token_escrow)
            .to_market(&self.to_market)
            .to_market_token_account(&self.to_market_token_escrow)
            .nonce(nonce)
            .bump(bumps.glv_shift)
            .params(params)
            .build()
            .execute()?;
        Ok(())
    }
}

impl<'info> internal::Authentication<'info> for CreateGlvShift<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
