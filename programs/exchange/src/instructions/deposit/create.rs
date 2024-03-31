use std::collections::BTreeSet;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount};
use data_store::{
    constants,
    cpi::accounts::{CheckRole, GetMarketMeta, GetTokenConfig, InitializeDeposit},
    program::DataStore,
    states::{deposit::TokenParams, NonceBytes},
    utils::Authentication,
};

use crate::ExchangeError;

/// Create Deposit Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateDepositParams {
    pub ui_fee_receiver: Pubkey,
    pub execution_fee: u64,
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
            ctx.accounts.token_transfer_ctx(true),
            params.initial_long_token_amount,
        )?;
    }

    if params.initial_short_token_amount != 0 {
        anchor_spl::token::transfer(
            ctx.accounts.token_transfer_ctx(false),
            params.initial_short_token_amount,
        )?;
    }

    let mut tokens = BTreeSet::default();
    tokens.insert(ctx.accounts.initial_long_token.mint);
    tokens.insert(ctx.accounts.initial_short_token.mint);

    let market_meta = cpi::get_market_meta(ctx.accounts.get_market_meta_ctx())?.get();
    tokens.insert(market_meta.long_token_mint);
    tokens.insert(market_meta.short_token_mint);

    // TODO: verify swap paths.
    let tokens_with_feed = tokens
        .into_iter()
        .map(|token| {
            let config = cpi::get_token_config(
                ctx.accounts.get_token_config_ctx(),
                ctx.accounts.store.key(),
                token,
            )?
            .get()
            .ok_or(ExchangeError::ResourceNotFound)?;
            Result::Ok((token, config.price_feed))
        })
        .collect::<Result<Vec<_>>>()?;

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
        tokens_with_feed,
    )?;
    // FIXME: should we allow using WNT to pay for the execution fee?
    require_gte!(
        ctx.accounts.deposit.lamports() + params.execution_fee,
        super::MAX_DEPOSIT_EXECUTION_FEE,
        ExchangeError::NotEnoughExecutionFee
    );
    if params.execution_fee != 0 {
        system_program::transfer(ctx.accounts.transfer_ctx(), params.execution_fee)?;
    }
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
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
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
    fn get_market_meta_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetMarketMeta<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetMarketMeta {
                market: self.market.to_account_info(),
            },
        )
    }

    fn get_token_config_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetTokenConfig<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetTokenConfig {
                map: self.token_config_map.to_account_info(),
            },
        )
    }

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

    fn token_transfer_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
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
            token::Transfer {
                from,
                to,
                authority: self.payer.to_account_info(),
            },
        )
    }

    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.deposit.to_account_info(),
            },
        )
    }
}
