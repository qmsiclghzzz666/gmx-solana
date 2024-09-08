use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    ops::{
        deposit::{CreateDepositOps, CreateDepositParams},
        execution_fee::TransferExecutionFeeOps,
    },
    states::{DepositV2, Market, NonceBytes, Store},
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
    pub initial_long_token: Box<Account<'info, Mint>>,
    /// initial short token.
    pub initial_short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving market tokens.
    #[account(
        init,
        payer = owner,
        associated_token::mint = market_token,
        associated_token::authority = deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        init,
        payer = owner,
        associated_token::mint = initial_long_token,
        associated_token::authority = deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        init,
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
    #[account(
        mut,
        has_one = store,
        constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched,
    )]
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
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    pub initial_long_token: Box<Account<'info, Mint>>,
    /// initial short token.
    pub initial_short_token: Box<Account<'info, Mint>>,
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
}

pub(crate) fn create_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateDeposit<'info>>,
    nonce: NonceBytes,
    params: &CreateDepositParams,
) -> Result<()> {
    let accounts = &ctx.accounts;
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
    fn transfer_tokens(&self, params: &CreateDepositParams) -> Result<()> {
        use anchor_spl::token::{transfer_checked, TransferChecked};

        if params.initial_long_token_amount != 0 {
            let Some(source) = self.initial_long_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_long_token_escrow.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            transfer_checked(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferChecked {
                        from: source.to_account_info(),
                        mint: self.initial_long_token.to_account_info(),
                        to: target.to_account_info(),
                        authority: self.owner.to_account_info(),
                    },
                ),
                params.initial_long_token_amount,
                params.initial_long_token_decimals,
            )?;
        }

        if params.initial_short_token_amount != 0 {
            let Some(source) = self.initial_short_token_source.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            let Some(target) = self.initial_short_token_escrow.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            transfer_checked(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferChecked {
                        from: source.to_account_info(),
                        mint: self.initial_short_token.to_account_info(),
                        to: target.to_account_info(),
                        authority: self.owner.to_account_info(),
                    },
                ),
                params.initial_short_token_amount,
                params.initial_short_token_decimals,
            )?;
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
