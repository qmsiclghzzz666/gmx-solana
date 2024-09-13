use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    events::RemoveDepositEvent,
    ops::{
        deposit::{CreateDepositOps, CreateDepositParams},
        execution_fee::TransferExecutionFeeOps,
    },
    states::{DepositV2, Market, NonceBytes, RoleKey, Store},
    utils::{
        internal::{self, Authentication},
        token::is_associated_token_account,
    },
    CoreError,
};

/// The accounts definition for the `prepare_deposit_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareDepositEscrow<'info> {
    /// The owner of the deposit.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The deposit owning these escrow accounts.
    /// CHECK: The deposit don't have to be initialized.
    #[account(
        seeds = [DepositV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: UncheckedAccount<'info>,
    /// Market token.
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// initial short token.
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The escrow account for receving market tokens.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = market_token,
        associated_token::authority = deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = initial_long_token,
        associated_token::authority = deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = initial_short_token,
        associated_token::authority = deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_deposit_escrow(
    _ctx: Context<PrepareDepositEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definition for the `create_deposit` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct CreateDeposit<'info> {
    /// The owner.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The deposit to be created.
    #[account(
        init,
        space = 8 + DepositV2::INIT_SPACE,
        payer = owner,
        seeds = [DepositV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: AccountLoader<'info, DepositV2>,
    /// Market token.
    #[account(constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// initial short token.
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The escrow account for receving market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA of the owner for receving market tokens.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = market_token,
        associated_token::authority = owner,
    )]
    pub market_token_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = initial_long_token)]
    /// The source initial long token account.
    pub initial_long_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The source initial short token account.
    #[account(mut, token::mint = initial_short_token)]
    pub initial_short_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn create_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateDeposit<'info>>,
    nonce: NonceBytes,
    params: &CreateDepositParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_fee(params)?;
    accounts.transfer_tokens(params)?;
    CreateDepositOps::builder()
        .deposit(accounts.deposit.clone())
        .market(accounts.market.clone())
        .store(accounts.store.clone())
        .owner(&accounts.owner)
        .nonce(&nonce)
        .bump(ctx.bumps.deposit)
        .initial_long_token(accounts.initial_long_token_escrow.as_deref())
        .initial_short_token(accounts.initial_short_token_escrow.as_deref())
        .market_token(&accounts.market_token_escrow)
        .params(params)
        .swap_paths(ctx.remaining_accounts)
        .build()
        .execute()?;
    Ok(())
}

impl<'info> CreateDeposit<'info> {
    fn transfer_tokens(&mut self, params: &CreateDepositParams) -> Result<()> {
        use anchor_spl::token::{transfer_checked, TransferChecked};

        if params.initial_long_token_amount != 0 {
            let Some(source) = self.initial_long_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_long_token_escrow.as_mut() else {
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
                params.initial_long_token_amount,
                mint.decimals,
            )?;
        }

        if params.initial_short_token_amount != 0 {
            let Some(source) = self.initial_short_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_short_token_escrow.as_mut() else {
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
                params.initial_short_token_amount,
                mint.decimals,
            )?;
        }

        // Make sure the data for escrow accounts is up-to-date.
        for escrow in self
            .initial_long_token_escrow
            .as_mut()
            .into_iter()
            .chain(self.initial_short_token_escrow.as_mut())
        {
            escrow.reload()?;
        }
        Ok(())
    }

    fn transfer_execution_fee(&self, params: &CreateDepositParams) -> Result<()> {
        TransferExecutionFeeOps::builder()
            .payment(self.deposit.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_fee)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }
}

/// The accounts definition for `close_deposit` instruction.
#[event_cpi]
#[derive(Accounts)]
pub struct CloseDeposit<'info> {
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
        constraint = deposit.load()?.tokens.market_token.token().expect("must exist") == market_token.key() @ CoreError::MarketTokenMintMismatched
    )]
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    #[account(
        constraint = deposit.load()?.tokens.initial_long_token.token().map(|token| initial_long_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// Initial short token.
    #[account(
        constraint = deposit.load()?.tokens.initial_short_token.token().map(|token| initial_short_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The deposit to close.
    #[account(
        mut,
        has_one = store,
        has_one = owner,
        constraint = deposit.load()?.tokens.market_token.account().expect("must exist") == market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = deposit.load()?.tokens.initial_long_token.account() == initial_long_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = deposit.load()?.tokens.initial_short_token.account() == initial_short_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        seeds = [DepositV2::SEED, store.key().as_ref(), owner.key().as_ref(), &deposit.load()?.nonce],
        bump = deposit.load()?.bump,
    )]
    pub deposit: AccountLoader<'info, DepositV2>,
    /// The escrow account for receving market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA for market token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(market_token_ata.key, owner.key, &market_token.key()) @ CoreError::NotAnATA,
    )]
    pub market_token_ata: UncheckedAccount<'info>,
    /// The ATA for inital long token of owner.
    /// CHECK: should be checked during the execution
    #[account(
        mut,
        constraint = is_associated_token_account(initial_long_token_ata.key, owner.key, &initial_long_token.as_ref().expect("must provided").key()) @ CoreError::NotAnATA,
    )]
    pub initial_long_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for inital short token of owner.
    /// CHECK: should be checked during the execution
    #[account(
        mut,
        constraint = is_associated_token_account(initial_short_token_ata.key, owner.key, &initial_short_token.as_ref().expect("must provided").key()) @ CoreError::NotAnATA,
    )]
    pub initial_short_token_ata: Option<UncheckedAccount<'info>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn close_deposit(ctx: Context<CloseDeposit>, reason: &str) -> Result<()> {
    let accounts = &ctx.accounts;
    let should_continue_when_atas_are_missing = accounts.preprocess()?;
    if accounts.transfer_to_atas(should_continue_when_atas_are_missing)? {
        {
            let deposit_address = accounts.deposit.key();
            let deposit = accounts.deposit.load()?;
            emit_cpi!(RemoveDepositEvent::new(
                deposit.id,
                deposit.store,
                deposit_address,
                deposit.tokens.market_token(),
                deposit.owner,
                deposit.action_state()?,
                reason,
            )?);
        }
        accounts.close()?;
    } else {
        msg!("Some ATAs are not initilaized, skip the close");
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for CloseDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.executor
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

type ShouldContinueWhenATAsAreMissing = bool;
type Success = bool;

impl<'info> CloseDeposit<'info> {
    fn preprocess(&self) -> Result<ShouldContinueWhenATAsAreMissing> {
        if self.executor.key == self.owner.key {
            Ok(true)
        } else {
            self.only_role(RoleKey::ORDER_KEEPER)?;
            {
                let deposit = self.deposit.load()?;
                if deposit.action_state()?.is_completed_or_cancelled() {
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
        let signer = self.deposit.load()?.signer();
        let seeds = signer.as_seeds();

        let builder = TransferAllFromEscrowToATA::builder()
            .system_program(self.system_program.to_account_info())
            .token_program(self.token_program.to_account_info())
            .associated_token_program(self.associated_token_program.to_account_info())
            .payer(self.executor.to_account_info())
            .owner(self.owner.to_account_info())
            .escrow_authority(self.deposit.to_account_info())
            .seeds(&seeds)
            .init_if_needed(init_if_needed);

        // Transfer market tokens.
        if !builder
            .clone()
            .mint(self.market_token.to_account_info())
            .ata(self.market_token_ata.to_account_info())
            .escrow(&self.market_token_escrow)
            .build()
            .execute()?
        {
            return Ok(false);
        }

        // Transfer initial long tokens.
        if let Some(escrow) = self.initial_long_token_escrow.as_ref() {
            let Some(ata) = self.initial_long_token_ata.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_long_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
            if !builder
                .clone()
                .mint(mint.to_account_info())
                .ata(ata.to_account_info())
                .escrow(escrow)
                .build()
                .execute()?
            {
                return Ok(false);
            }
        }

        // Transfer initial short tokens.
        if let Some(escrow) = self.initial_short_token_escrow.as_ref() {
            let Some(ata) = self.initial_short_token_ata.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(mint) = self.initial_short_token.as_ref() else {
                return err!(CoreError::MintAccountNotProvided);
            };
            if !builder
                .clone()
                .mint(mint.to_account_info())
                .ata(ata.to_account_info())
                .escrow(escrow)
                .build()
                .execute()?
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn close(&self) -> Result<()> {
        self.deposit.close(self.owner.to_account_info())?;
        Ok(())
    }
}
