use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};
use data_store::{
    constants,
    cpi::accounts::{CheckRole, InitializeDeposit},
    program::DataStore,
    states::{deposit::TokenParams, Market, NonceBytes},
    utils::Authentication,
};

use crate::ExchangeError;

/// Create Deposit Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateDepositParams {
    pub ui_fee_receiver: Pubkey,
    pub long_token_swap_path: Vec<Pubkey>,
    pub short_token_swap_path: Vec<Pubkey>,
    pub initial_long_token_amount: u64,
    pub initial_short_token_amount: u64,
    pub min_market_token: u64,
    pub should_unwrap_native_token: bool,
}

/// Create Deposit.
pub fn create_deposit(
    ctx: Context<CreateDeposit>,
    nonce: NonceBytes,
    params: CreateDepositParams,
) -> Result<()> {
    use data_store::cpi;

    require!(
        params.initial_long_token_amount != 0 || params.initial_short_token_amount != 0,
        ExchangeError::EmptyDepositAmounts
    );

    if params.initial_long_token_amount != 0 {
        anchor_spl::token::transfer(
            ctx.accounts.transfer_ctx(true),
            params.initial_long_token_amount,
        )?;
    }

    if params.initial_short_token_amount != 0 {
        anchor_spl::token::transfer(
            ctx.accounts.transfer_ctx(false),
            params.initial_short_token_amount,
        )?;
    }

    // TODO: allow using WNT to pay for the execution fee.

    cpi::initialize_deposit(
        ctx.accounts.initialize_deposit_ctx(),
        nonce,
        params.ui_fee_receiver,
        TokenParams {
            initial_long_token: ctx.accounts.initial_long_token.mint,
            initial_short_token: ctx.accounts.initial_short_token.mint,
            long_token_swap_path: params.long_token_swap_path,
            short_token_swap_path: params.short_token_swap_path,
            initial_long_token_amount: params.initial_long_token_amount,
            initial_short_token_amount: params.initial_short_token_amount,
            min_market_tokens: params.min_market_token,
            should_unwrap_native_token: params.should_unwrap_native_token,
        },
    )?;
    // TODO: check the execution fee.
    // TODO: emit deposit created event.
    Ok(())
}

#[derive(Accounts)]
pub struct CreateDeposit<'info> {
    pub authority: Signer<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_controller: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: only used to invoke CPI which will then initialize the account.
    #[account(mut)]
    pub deposit: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub receiver: Account<'info, TokenAccount>,
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub initial_long_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initial_short_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = initial_long_token.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            long_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub long_token_deposit_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = initial_short_token.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            short_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub short_token_deposit_vault: Account<'info, TokenAccount>,
}

impl<'info> Authentication<'info> for CreateDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            CheckRole {
                store: self.store.to_account_info(),
                roles: self.only_controller.to_account_info(),
            },
        )
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> CreateDeposit<'info> {
    fn initialize_deposit_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitializeDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeDeposit {
                authority: self.authority.to_account_info(),
                payer: self.payer.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                market: self.market.to_account_info(),
                receiver: self.receiver.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn transfer_ctx(&self, is_long_token: bool) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let (from, to) = if is_long_token {
            (
                self.initial_long_token.to_account_info(),
                self.long_token_deposit_vault.to_account_info(),
            )
        } else {
            (
                self.initial_short_token.to_account_info(),
                self.short_token_deposit_vault.to_account_info(),
            )
        };
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from,
                to,
                authority: self.payer.to_account_info(),
            },
        )
    }
}
