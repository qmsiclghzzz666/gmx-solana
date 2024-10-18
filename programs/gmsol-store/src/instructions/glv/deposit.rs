use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_2022::Token2022,
    token_interface,
};
use gmsol_utils::InitSpace;

use crate::{
    constants,
    events::RemoveGlvDepositEvent,
    ops::{
        execution_fee::{PayExecutionFeeOperation, TransferExecutionFeeOperation},
        glv::{CreateGlvDepositOperation, CreateGlvDepositParams},
        market::{MarketTransferInOperation, MarketTransferOutOperation},
    },
    states::{
        common::action::{ActionExt, ActionSigner},
        Glv, GlvDeposit, Market, NonceBytes, Oracle, PriceProvider, RoleKey, Seed, Store,
        TokenMapHeader,
    },
    utils::{
        internal::{self, Authentication},
        token::is_associated_token_account,
    },
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

/// The accounts definition for `close_glv_deposit` instruction.
#[event_cpi]
#[derive(Accounts)]
pub struct CloseGlvDeposit<'info> {
    /// The executor of this instruction.
    pub executor: Signer<'info>,
    /// The store.
    pub store: AccountLoader<'info, Store>,
    /// The owner of the deposit.
    /// CHECK: only use to validate and receive fund.
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    /// Market token.
    #[account(
        constraint = glv_deposit.load()?.tokens.market_token() == market_token.key() @ CoreError::MarketTokenMintMismatched
    )]
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    #[account(
        constraint = glv_deposit.load()?.tokens.initial_long_token.token().map(|token| initial_long_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// Initial short token.
    #[account(
        constraint = glv_deposit.load()?.tokens.initial_short_token.token().map(|token| initial_short_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// GLV token.
    #[account(
        constraint = glv_deposit.load()?.tokens.glv_token() == glv_token.key() @ CoreError::TokenMintMismatched
    )]
    pub glv_token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    /// The GLV deposit to close.
    #[account(
        mut,
        constraint = glv_deposit.load()?.header.owner == owner.key() @ CoreError::OwnerMismatched,
        constraint = glv_deposit.load()?.header.store == store.key() @ CoreError::StoreMismatched,
        constraint = glv_deposit.load()?.tokens.market_token_account() == market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.glv_token_account() == glv_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.initial_long_token.account() == initial_long_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.initial_short_token.account() == initial_short_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        seeds = [GlvDeposit::SEED, store.key().as_ref(), owner.key().as_ref(), &glv_deposit.load()?.header.nonce],
        bump = glv_deposit.load()?.header.bump,
    )]
    pub glv_deposit: AccountLoader<'info, GlvDeposit>,
    /// The escrow account for market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = glv_deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for GLV tokens.
    #[account(
        mut,
        associated_token::mint = glv_token,
        associated_token::authority = glv_deposit,
        associated_token::token_program = glv_token_program,
    )]
    pub glv_token_escrow: Box<InterfaceAccount<'info, token_interface::TokenAccount>>,
    /// The ATA for market token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(market_token_ata.key, owner.key, &market_token.key()) @ CoreError::NotAnATA,
    )]
    pub market_token_ata: UncheckedAccount<'info>,
    /// The ATA for initial long token of owner.
    /// CHECK: should be checked during the execution
    #[account(
        mut,
        constraint = is_associated_token_account(initial_long_token_ata.key, owner.key, &initial_long_token.as_ref().expect("must provided").key()) @ CoreError::NotAnATA,
    )]
    pub initial_long_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for initial short token of owner.
    /// CHECK: should be checked during the execution
    #[account(
        mut,
        constraint = is_associated_token_account(initial_short_token_ata.key, owner.key, &initial_short_token.as_ref().expect("must provided").key()) @ CoreError::NotAnATA,
    )]
    pub initial_short_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for GLV token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(glv_token_ata.key, owner.key, &glv_token.key()) @ CoreError::NotAnATA,
    )]
    pub glv_token_ata: UncheckedAccount<'info>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// Token program for GLV token.
    pub glv_token_program: Program<'info, Token2022>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn close_glv_deposit(ctx: Context<CloseGlvDeposit>, reason: &str) -> Result<()> {
    let accounts = &ctx.accounts;
    let should_continue_when_atas_are_missing = accounts.preprocess()?;
    if accounts.transfer_to_atas(should_continue_when_atas_are_missing)? {
        {
            let glv_deposit_address = accounts.glv_deposit.key();
            let glv_deposit = accounts.glv_deposit.load()?;
            emit_cpi!(RemoveGlvDepositEvent::new(
                glv_deposit.header.id,
                glv_deposit.header.store,
                glv_deposit_address,
                glv_deposit.tokens.market_token(),
                glv_deposit.tokens.glv_token(),
                glv_deposit.header.owner,
                glv_deposit.header.action_state()?,
                reason,
            )?);
        }
        accounts.close()?;
    } else {
        msg!("Some ATAs are not initilaized, skip the close");
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for CloseGlvDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.executor
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

type ShouldContinueWhenATAsAreMissing = bool;
type Success = bool;

impl<'info> CloseGlvDeposit<'info> {
    fn preprocess(&self) -> Result<ShouldContinueWhenATAsAreMissing> {
        if self.executor.key == self.owner.key {
            Ok(true)
        } else {
            self.only_role(RoleKey::ORDER_KEEPER)?;
            {
                let glv_deposit = self.glv_deposit.load()?;
                if glv_deposit
                    .header
                    .action_state()?
                    .is_completed_or_cancelled()
                {
                    Ok(false)
                } else {
                    err!(CoreError::PermissionDenied)
                }
            }
        }
    }

    fn transfer_to_atas(&self, init_if_needed: bool) -> Result<Success> {
        use crate::utils::token::TransferAllFromEscrowToATA;

        // Prepare signer seeds.
        let signer = self.glv_deposit.load()?.signer();
        let seeds = signer.as_seeds();

        let builder = TransferAllFromEscrowToATA::builder()
            .system_program(self.system_program.to_account_info())
            .associated_token_program(self.associated_token_program.to_account_info())
            .payer(self.executor.to_account_info())
            .owner(self.owner.to_account_info())
            .escrow_authority(self.glv_deposit.to_account_info())
            .seeds(&seeds)
            .init_if_needed(init_if_needed);

        // Transfer market tokens.
        if !builder
            .clone()
            .token_program(self.token_program.to_account_info())
            .mint(self.market_token.to_account_info())
            .decimals(self.market_token.decimals)
            .ata(self.market_token_ata.to_account_info())
            .escrow(self.market_token_escrow.to_account_info())
            .build()
            .execute()?
        {
            return Ok(false);
        }

        // Transfer GLV tokens.
        if !builder
            .clone()
            .token_program(self.glv_token_program.to_account_info())
            .mint(self.glv_token.to_account_info())
            .decimals(self.glv_token.decimals)
            .ata(self.glv_token_ata.to_account_info())
            .escrow(self.glv_token_escrow.to_account_info())
            .build()
            .execute()?
        {
            return Ok(false);
        }

        // Prevent closing the same token accounts.
        let (initial_long_token_escrow, initial_short_token_escrow) =
            if self.initial_long_token_escrow.as_ref().map(|a| a.key())
                == self.initial_short_token_escrow.as_ref().map(|a| a.key())
            {
                (self.initial_long_token_escrow.as_ref(), None)
            } else {
                (
                    self.initial_long_token_escrow.as_ref(),
                    self.initial_short_token_escrow.as_ref(),
                )
            };

        // Transfer initial long tokens.
        if let Some(escrow) = initial_long_token_escrow.as_ref() {
            let Some(ata) = self.initial_long_token_ata.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_long_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
            if !builder
                .clone()
                .token_program(self.token_program.to_account_info())
                .mint(mint.to_account_info())
                .decimals(mint.decimals)
                .ata(ata.to_account_info())
                .escrow(escrow.to_account_info())
                .build()
                .execute()?
            {
                return Ok(false);
            }
        }

        // Transfer initial short tokens.
        if let Some(escrow) = initial_short_token_escrow.as_ref() {
            let Some(ata) = self.initial_short_token_ata.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_short_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
            if !builder
                .clone()
                .token_program(self.token_program.to_account_info())
                .mint(mint.to_account_info())
                .decimals(mint.decimals)
                .ata(ata.to_account_info())
                .escrow(escrow.to_account_info())
                .build()
                .execute()?
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn close(&self) -> Result<()> {
        self.glv_deposit.close(self.owner.to_account_info())?;
        Ok(())
    }
}

/// The accounts definition for `execute_glv_deposit` instruction.
#[derive(Accounts)]
pub struct ExecuteGlvDeposit<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// Token Map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Price Provider.
    pub price_provider: Interface<'info, PriceProvider>,
    /// Oracle buffer to use.
    #[account(has_one = store)]
    pub oracle: Box<Account<'info, Oracle>>,
    /// GLV account.
    #[account(
        has_one = store,
        constraint = glv.load()?.market_tokens().contains(&market_token.key()) @ CoreError::InvalidArgument,
    )]
    pub glv: AccountLoader<'info, Glv>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The GLV deposit to execute.
    #[account(
        mut,
        constraint = glv_deposit.load()?.header.store == store.key() @ CoreError::StoreMismatched,
        constraint = glv_deposit.load()?.header.market == market.key() @ CoreError::MarketMismatched,
        constraint = glv_deposit.load()?.tokens.glv_token() == glv_token.key() @ CoreError::TokenMintMismatched,
        constraint = glv_deposit.load()?.tokens.market_token() == market_token.key() @ CoreError::MarketTokenMintMismatched,
        constraint = glv_deposit.load()?.tokens.market_token_account() == market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.glv_token_account() == glv_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.initial_long_token.account() == initial_long_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = glv_deposit.load()?.tokens.initial_short_token.account() == initial_short_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        seeds = [GlvDeposit::SEED, store.key().as_ref(), glv_deposit.load()?.header.owner.as_ref(), &glv_deposit.load()?.header.nonce],
        bump = glv_deposit.load()?.header.bump,
    )]
    pub glv_deposit: AccountLoader<'info, GlvDeposit>,
    /// GLV token mint.
    #[account(mut, constraint = glv.load()?.glv_token == glv_token.key() @ CoreError::TokenMintMismatched)]
    pub glv_token: Box<InterfaceAccount<'info, token_interface::Mint>>,
    /// Market token mint.
    #[account(mut, constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    #[account(
        constraint = glv_deposit.load()?.tokens.initial_long_token.token().map(|token| initial_long_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// Initial short token.
    #[account(
        constraint = glv_deposit.load()?.tokens.initial_short_token.token().map(|token| initial_short_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The escrow account for GLV tokens.
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
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = glv_deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// Initial long token vault.
    #[account(
        mut,
        token::mint = initial_long_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_long_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub initial_long_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    /// Initial short token vault.
    #[account(
        mut,
        token::mint = initial_short_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_short_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub initial_short_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    /// Market token vault for the GLV.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = glv,
    )]
    pub market_token_vault: Box<Account<'info, TokenAccount>>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The token program for GLV token.
    pub glv_token_program: Program<'info, Token2022>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// CHECK: only ORDER_KEEPER is allowed to call this function.
pub(crate) fn unchecked_execute_glv_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteGlvDeposit<'info>>,
    execution_lamports: u64,
    throw_on_execution_error: bool,
) -> Result<()> {
    let accounts = ctx.accounts;
    let remaining_accounts = ctx.remaining_accounts;
    let signer = accounts.glv_deposit.load()?.signer();

    accounts.transfer_tokens_in(&signer, remaining_accounts)?;

    let executed = accounts.perform_execution(remaining_accounts, throw_on_execution_error)?;

    if executed {
        accounts.glv_deposit.load_mut()?.header.completed()?;
    } else {
        accounts.glv_deposit.load_mut()?.header.cancelled()?;
        accounts.transfer_tokens_out(remaining_accounts)?;
    }

    // It must be placed at the end to be executed correctly.
    accounts.pay_execution_fee(execution_lamports)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for ExecuteGlvDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ExecuteGlvDeposit<'info> {
    #[inline(never)]
    fn pay_execution_fee(&self, execution_fee: u64) -> Result<()> {
        let execution_lamports = self.glv_deposit.load()?.execution_lamports(execution_fee);
        PayExecutionFeeOperation::builder()
            .payer(self.glv_deposit.to_account_info())
            .receiver(self.authority.to_account_info())
            .execution_lamports(execution_lamports)
            .build()
            .execute()?;
        Ok(())
    }

    #[inline(never)]
    fn transfer_tokens_in(
        &self,
        signer: &ActionSigner,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        self.transfer_market_tokens_in(signer)?;
        self.transfer_initial_tokens_in(signer, remaining_accounts)?;
        Ok(())
    }

    #[inline(never)]
    fn transfer_tokens_out(&self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        self.transfer_market_tokens_out()?;
        self.transfer_initial_tokens_out(remaining_accounts)?;
        Ok(())
    }

    fn transfer_market_tokens_in(&self, signer: &ActionSigner) -> Result<()> {
        use anchor_spl::token_interface::{transfer_checked, TransferChecked};

        let amount = self.glv_deposit.load()?.params.market_token_amount;
        if amount != 0 {
            let token = &self.market_token;
            let from = &self.market_token_escrow;
            let to = &self.market_token_vault;
            let ctx = CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: from.to_account_info(),
                    mint: token.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.glv_deposit.to_account_info(),
                },
            );
            transfer_checked(
                ctx.with_signer(&[&signer.as_seeds()]),
                amount,
                token.decimals,
            )?;
        }
        Ok(())
    }

    fn transfer_market_tokens_out(&self) -> Result<()> {
        use anchor_spl::token_interface::{transfer_checked, TransferChecked};

        let amount = self.glv_deposit.load()?.params.market_token_amount;
        if amount != 0 {
            let token = &self.market_token;
            let from = &self.market_token_vault;
            let to = &self.market_token_escrow;
            let ctx = CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: from.to_account_info(),
                    mint: token.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.glv.to_account_info(),
                },
            );
            let glv = self.glv.load()?;
            transfer_checked(
                ctx.with_signer(&[&glv.signer_seeds()]),
                amount,
                token.decimals,
            )?;
        }
        Ok(())
    }

    fn transfer_initial_tokens_in(
        &self,
        sigenr: &ActionSigner,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        let seeds = sigenr.as_seeds();
        let builder = MarketTransferInOperation::builder()
            .store(&self.store)
            .from_authority(self.glv_deposit.to_account_info())
            .token_program(self.token_program.to_account_info())
            .signer_seeds(&seeds);
        let store = &self.store.key();

        for is_primary in [true, false] {
            let (amount, escrow, vault) = if is_primary {
                (
                    self.glv_deposit.load()?.params.initial_long_token_amount,
                    self.initial_long_token_escrow.as_ref(),
                    self.initial_long_token_vault.as_ref(),
                )
            } else {
                (
                    self.glv_deposit.load()?.params.initial_short_token_amount,
                    self.initial_short_token_escrow.as_ref(),
                    self.initial_short_token_vault.as_ref(),
                )
            };

            if amount == 0 {
                continue;
            }

            let escrow = escrow.ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let market = self
                .glv_deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, is_primary, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = vault.ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .from(escrow.to_account_info())
                .vault(vault)
                .amount(amount)
                .build()
                .execute()?;
        }

        Ok(())
    }

    fn transfer_initial_tokens_out(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        let builder = MarketTransferOutOperation::builder()
            .store(&self.store)
            .token_program(self.token_program.to_account_info());

        let store = &self.store.key();

        for is_primary in [true, false] {
            let (amount, token, escrow, vault) = if is_primary {
                (
                    self.glv_deposit.load()?.params.initial_long_token_amount,
                    self.initial_long_token.as_ref(),
                    self.initial_long_token_escrow.as_ref(),
                    self.initial_long_token_vault.as_ref(),
                )
            } else {
                (
                    self.glv_deposit.load()?.params.initial_short_token_amount,
                    self.initial_short_token.as_ref(),
                    self.initial_short_token_escrow.as_ref(),
                    self.initial_short_token_vault.as_ref(),
                )
            };

            let Some(escrow) = escrow else {
                continue;
            };

            let market = self
                .glv_deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, is_primary, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let token = token.ok_or(error!(CoreError::TokenMintNotProvided))?;
            let vault = vault.ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .to(escrow.to_account_info())
                .vault(vault.to_account_info())
                .amount(amount)
                .decimals(token.decimals)
                .token_mint(token.to_account_info())
                .build()
                .execute()?;
        }

        Ok(())
    }

    fn perform_execution(
        &self,
        _remaining_accounts: &'info [AccountInfo<'info>],
        _throw_on_execution_error: bool,
    ) -> Result<bool> {
        todo!()
    }
}
