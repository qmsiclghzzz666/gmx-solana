use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use gmsol_utils::InitSpace;

use crate::{
    ops::shift::{CreateShiftOperation, CreateShiftParams},
    states::{
        common::action::{self, Action, ActionExt},
        glv::GlvShift,
        Glv, Market, NonceBytes, RoleKey, Seed, Store,
    },
    utils::internal,
    CoreError,
};

/// The accounts definition for [`create_glv_shift`] instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateGlvShift<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// GLV.
    #[account(
        mut,
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
        payer = glv,
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
        self.glv.to_account_info()
    }

    fn payer_seeds(&self) -> Result<Option<Vec<Vec<u8>>>> {
        Ok(Some(self.glv.load()?.vec_signer_seeds()))
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

/// The accounts definition for [`close_glv_shift`] instruction.
#[event_cpi]
#[derive(Accounts)]
pub struct CloseGlvShift<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The store.
    pub store: AccountLoader<'info, Store>,
    /// GLV.
    #[account(
        has_one = store,
        constraint = glv.load()?.contains(&from_market_token.key()) @ CoreError::InvalidArgument,
        constraint = glv.load()?.contains(&to_market_token.key()) @ CoreError::InvalidArgument,
    )]
    pub glv: AccountLoader<'info, Glv>,
    /// The GLV shift to close.
    #[account(
        mut,
        constraint = glv_shift.load()?.header().owner == glv.key() @ CoreError::OwnerMismatched,
        constraint = glv_shift.load()?.header().store == store.key() @ CoreError::StoreMismatched,
        constraint = glv_shift.load()?.tokens().from_market_token_account() == from_market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = glv_shift.load()?.tokens().to_market_token_account() == to_market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        seeds = [GlvShift::SEED, store.key().as_ref(), glv.key().as_ref(), &glv_shift.load()?.header().nonce],
        bump = glv_shift.load()?.header().bump,
    )]
    pub glv_shift: AccountLoader<'info, GlvShift>,
    /// From Market token.
    #[account(
        constraint = glv_shift.load()?.tokens().from_market_token() == from_market_token.key() @ CoreError::MarketTokenMintMismatched
    )]
    pub from_market_token: Box<Account<'info, Mint>>,
    /// To Market token.
    #[account(
        constraint = glv_shift.load()?.tokens().to_market_token() == to_market_token.key() @ CoreError::MarketTokenMintMismatched
    )]
    pub to_market_token: Box<Account<'info, Mint>>,
    /// The escrow account for from market tokens.
    #[account(
        mut,
        associated_token::mint = from_market_token,
        associated_token::authority = glv_shift,
    )]
    pub from_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for to market tokens.
    #[account(
        mut,
        associated_token::mint = to_market_token,
        associated_token::authority = glv_shift,
    )]
    pub to_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// Vault for from market tokens.
    #[account(
        mut,
        associated_token::mint = from_market_token,
        associated_token::authority = glv,
    )]
    pub from_market_token_vault: Box<Account<'info, TokenAccount>>,
    /// Vault for to market tokens.
    #[account(
        mut,
        associated_token::mint = to_market_token,
        associated_token::authority = glv,
    )]
    pub to_market_token_vault: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> action::Close<'info, GlvShift> for CloseGlvShift<'info> {
    fn expected_role(&self) -> &str {
        RoleKey::ORDER_KEEPER
    }

    fn fund_receiver(&self) -> AccountInfo<'info> {
        self.glv.to_account_info()
    }

    fn transfer_to_atas(&self, init_if_needed: bool) -> Result<action::TransferSuccess> {
        use crate::utils::token::TransferAllFromEscrowToATA;

        let signer = self.glv_shift.load()?.signer();
        let seeds = signer.as_seeds();

        let builder = TransferAllFromEscrowToATA::builder()
            .system_program(self.system_program.to_account_info())
            .token_program(self.token_program.to_account_info())
            .associated_token_program(self.associated_token_program.to_account_info())
            .payer(self.authority.to_account_info())
            .owner(self.glv.to_account_info())
            .escrow_authority(self.glv_shift.to_account_info())
            .seeds(&seeds)
            .init_if_needed(init_if_needed);

        // Transfer from market tokens.
        if !builder
            .clone()
            .mint(self.from_market_token.to_account_info())
            .decimals(self.from_market_token.decimals)
            .ata(self.from_market_token_vault.to_account_info())
            .escrow(self.from_market_token_escrow.to_account_info())
            .build()
            .execute()?
        {
            return Ok(false);
        }

        // Transfer to market tokens.
        if !builder
            .clone()
            .mint(self.to_market_token.to_account_info())
            .decimals(self.to_market_token.decimals)
            .ata(self.to_market_token_vault.to_account_info())
            .escrow(self.to_market_token_escrow.to_account_info())
            .build()
            .execute()?
        {
            return Ok(false);
        }

        Ok(true)
    }

    fn event_authority(&self, bumps: &Self::Bumps) -> (AccountInfo<'info>, u8) {
        (self.event_authority.clone(), bumps.event_authority)
    }

    fn action(&self) -> &AccountLoader<'info, GlvShift> {
        &self.glv_shift
    }
}

impl<'info> internal::Authentication<'info> for CloseGlvShift<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
