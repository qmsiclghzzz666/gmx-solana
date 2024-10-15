use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};
use gmsol_utils::InitSpace;

use crate::{
    events::RemoveShiftEvent,
    ops::{
        execution_fee::TransferExecutionFeeOperation,
        shift::{CreateShiftOperation, CreateShiftParams},
    },
    states::{common::action::ActionExt, Market, NonceBytes, RoleKey, Seed, Shift, Store},
    utils::{
        internal::{self, Authentication},
        token::is_associated_token_account,
    },
    CoreError,
};

/// The accounts definitions for the `prepare_shift_escorw` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareShiftEscorw<'info> {
    /// The owner of the shift.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The shift account owning these escrow accoutns.
    /// CHECK: The shift account don't have to be initialized.
    #[account(
        seeds = [Shift::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub shift: UncheckedAccount<'info>,
    /// From market token.
    pub from_market_token: Box<Account<'info, Mint>>,
    /// To Market token.
    pub to_market_token: Box<Account<'info, Mint>>,
    /// The escrow account for from market tokens.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = from_market_token,
        associated_token::authority = shift,
    )]
    pub from_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for to market tokens.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = to_market_token,
        associated_token::authority = shift,
    )]
    pub to_market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_shift_escrow(
    _ctx: Context<PrepareShiftEscorw>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

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
    pub to_market_token_ata: Box<Account<'info, TokenAccount>>,
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
    CreateShiftOperation::builder()
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
        TransferExecutionFeeOperation::builder()
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

/// The accounts definition for the `close_shift` instruction.
#[event_cpi]
#[derive(Accounts)]
pub struct CloseShift<'info> {
    /// The executor of this instruction.
    pub executor: Signer<'info>,
    /// The store.
    pub store: AccountLoader<'info, Store>,
    /// The owenr of the shift.
    /// CHECK: only use to validate and receive fund.
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    /// The shift to close.
    #[account(
        mut,
        constraint = shift.load()?.header.store == store.key() @ CoreError::StoreMismatched,
        constraint = shift.load()?.header.owner == owner.key() @ CoreError::OwnerMismatched,
        constraint = shift.load()?.tokens.from_market_token_account() == from_market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = shift.load()?.tokens.to_market_token_account() == to_market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
    )]
    pub shift: AccountLoader<'info, Shift>,
    /// From market token.
    #[account(constraint = shift.load()?.tokens.from_market_token() == from_market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub from_market_token: Box<Account<'info, Mint>>,
    /// To market token.
    #[account(constraint = shift.load()?.tokens.to_market_token() == to_market_token.key() @ CoreError::MarketTokenMintMismatched)]
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
    /// The ATA for from market token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(from_market_token_ata.key, owner.key, &from_market_token.key()) @ CoreError::NotAnATA,
    )]
    pub from_market_token_ata: UncheckedAccount<'info>,
    /// The ATA for to market token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(to_market_token_ata.key, owner.key, &to_market_token.key()) @ CoreError::NotAnATA,
    )]
    pub to_market_token_ata: UncheckedAccount<'info>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn close_shift(ctx: Context<CloseShift>, reason: &str) -> Result<()> {
    let accounts = &ctx.accounts;
    let should_continue_when_atas_are_missing = accounts.preprocess()?;
    if accounts.transfer_to_atas(should_continue_when_atas_are_missing)? {
        {
            let shift_address = accounts.shift.key();
            let shift = accounts.shift.load()?;
            emit_cpi!(RemoveShiftEvent::new(
                shift.header.id,
                shift.header.store,
                shift_address,
                shift.tokens().from_market_token(),
                shift.header.owner,
                shift.header.action_state()?,
                reason,
            )?)
        }
        accounts.close()?;
    } else {
        msg!("Some ATAs are not initilaized, skip the close");
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for CloseShift<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.executor
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

type ShouldContinueWhenATAsAreMissing = bool;
type Success = bool;

impl<'info> CloseShift<'info> {
    fn preprocess(&self) -> Result<ShouldContinueWhenATAsAreMissing> {
        if self.executor.key == self.owner.key {
            Ok(true)
        } else {
            self.only_role(RoleKey::ORDER_KEEPER)?;
            {
                let shift = self.shift.load()?;
                if shift.header.action_state()?.is_completed_or_cancelled() {
                    Ok(false)
                } else {
                    err!(CoreError::PermissionDenied)
                }
            }
        }
    }

    fn transfer_to_atas(&self, init_if_needed: bool) -> Result<Success> {
        use crate::utils::token::TransferAllFromEscrowToATA;

        let signer = self.shift.load()?.signer();
        let seeds = signer.as_seeds();

        let builder = TransferAllFromEscrowToATA::builder()
            .system_program(self.system_program.to_account_info())
            .token_program(self.token_program.to_account_info())
            .associated_token_program(self.associated_token_program.to_account_info())
            .payer(self.executor.to_account_info())
            .owner(self.owner.to_account_info())
            .escrow_authority(self.shift.to_account_info())
            .seeds(&seeds)
            .init_if_needed(init_if_needed);

        // Transfer from market tokens.
        if !builder
            .clone()
            .mint(self.from_market_token.to_account_info())
            .ata(self.from_market_token_ata.to_account_info())
            .escrow(&self.from_market_token_escrow)
            .build()
            .execute()?
        {
            return Ok(false);
        }

        // Transfer to market tokens.
        if !builder
            .clone()
            .mint(self.to_market_token.to_account_info())
            .ata(self.to_market_token_ata.to_account_info())
            .escrow(&self.to_market_token_escrow)
            .build()
            .execute()?
        {
            return Ok(false);
        }

        Ok(true)
    }

    fn close(&self) -> Result<()> {
        self.shift.close(self.owner.to_account_info())?;
        Ok(())
    }
}
