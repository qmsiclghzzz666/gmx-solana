use std::collections::BTreeSet;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Token, TokenAccount};
use data_store::{
    cpi::accounts::{GetTokenConfig, GetValidatedMarketMeta, InitializeWithdrawal},
    program::DataStore,
    states::{
        common::{SwapParams, TokenRecord},
        withdrawal::TokenParams,
        NonceBytes,
    },
};

use crate::{
    events::WithdrawalCreatedEvent,
    utils::{market::get_and_validate_swap_path, ControllerSeeds},
    ExchangeError,
};

/// Create Withdrawal Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateWithdrawalParams {
    pub market_token_amount: u64,
    pub execution_fee: u64,
    pub ui_fee_receiver: Pubkey,
    pub tokens: TokenParams,
    pub long_token_swap_length: u8,
    pub short_token_swap_length: u8,
}

#[derive(Accounts)]
pub struct CreateWithdrawal<'info> {
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_controller: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI which will then initalize the account.
    #[account(mut)]
    pub withdrawal: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Market token account from which funds are burnt.
    ///
    /// ## Notes
    /// - The mint of this account is checked to be the same as the vault,
    /// but whether to be matched the market token mint of the `market` should be checked by
    /// [`get_market_token_mint`](data_store::instructions::get_market_token_mint) through CPI.
    #[account(mut)]
    pub market_token_account: Account<'info, TokenAccount>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market_token_withdrawal_vault: UncheckedAccount<'info>,
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    pub final_short_token_receiver: Account<'info, TokenAccount>,
}

/// Create withdrawal.
pub fn create_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateWithdrawal<'info>>,
    nonce: NonceBytes,
    params: CreateWithdrawalParams,
) -> Result<()> {
    use data_store::cpi;

    require!(
        params.market_token_amount != 0,
        ExchangeError::EmptyWithdrawalAmount
    );

    let mut tokens = BTreeSet::default();
    tokens.insert(ctx.accounts.final_long_token_receiver.mint);
    tokens.insert(ctx.accounts.final_short_token_receiver.mint);

    // The market token mint used to withdraw must match the `market`'s.
    let market_meta =
        cpi::get_validated_market_meta(ctx.accounts.get_validated_market_meta_ctx())?.get();
    require_eq!(
        ctx.accounts.market_token_account.mint,
        market_meta.market_token_mint,
        ExchangeError::MismatchedMarketTokenMint
    );
    tokens.insert(market_meta.index_token_mint);
    tokens.insert(market_meta.long_token_mint);
    tokens.insert(market_meta.short_token_mint);

    // Handle the swap paths.
    let long_swap_length = params.long_token_swap_length as usize;
    let short_swap_length = params.short_token_swap_length as usize;
    require_gte!(
        ctx.remaining_accounts.len(),
        long_swap_length + short_swap_length,
        ExchangeError::NotEnoughRemainingAccounts,
    );
    let long_token_swap_path = get_and_validate_swap_path(
        &ctx.accounts.data_store_program,
        ctx.accounts.store.to_account_info(),
        &ctx.remaining_accounts[..long_swap_length],
        &market_meta.long_token_mint,
        &ctx.accounts.final_long_token_receiver.mint,
        &mut tokens,
    )?;
    let short_token_swap_path = get_and_validate_swap_path(
        &ctx.accounts.data_store_program,
        ctx.accounts.store.to_account_info(),
        &ctx.remaining_accounts[long_swap_length..(long_swap_length + short_swap_length)],
        &market_meta.short_token_mint,
        &ctx.accounts.final_short_token_receiver.mint,
        &mut tokens,
    )?;

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
            TokenRecord::from_config(token, &config)
        })
        .collect::<Result<Vec<_>>>()?;

    let controller = ControllerSeeds::new(ctx.accounts.store.key, ctx.bumps.authority);
    cpi::initialize_withdrawal(
        ctx.accounts
            .initialize_withdrawal_ctx()
            .with_signer(&[&controller.as_seeds()]),
        nonce,
        SwapParams {
            long_token_swap_path,
            short_token_swap_path,
        },
        tokens_with_feed,
        params.tokens,
        params.market_token_amount,
        params.ui_fee_receiver,
    )?;

    require_gte!(
        ctx.accounts.withdrawal.lamports() + params.execution_fee,
        super::MAX_WITHDRAWAL_EXECUTION_FEE,
        ExchangeError::NotEnoughExecutionFee
    );
    if params.execution_fee != 0 {
        system_program::transfer(ctx.accounts.transfer_ctx(), params.execution_fee)?;
    }
    emit!(WithdrawalCreatedEvent {
        ts: Clock::get()?.unix_timestamp,
        store: ctx.accounts.store.key(),
        withdrawal: ctx.accounts.withdrawal.key(),
    });
    Ok(())
}

impl<'info> CreateWithdrawal<'info> {
    fn get_token_config_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetTokenConfig<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetTokenConfig {
                map: self.token_config_map.to_account_info(),
            },
        )
    }

    fn get_validated_market_meta_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, GetValidatedMarketMeta<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetValidatedMarketMeta {
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
            },
        )
    }

    fn initialize_withdrawal_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeWithdrawal {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                payer: self.payer.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                market_token_account: self.market_token_account.to_account_info(),
                market: self.market.to_account_info(),
                final_long_token_receiver: self.final_long_token_receiver.to_account_info(),
                final_short_token_receiver: self.final_short_token_receiver.to_account_info(),
                system_program: self.system_program.to_account_info(),
                market_token_withdrawal_vault: self.market_token_withdrawal_vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.withdrawal.to_account_info(),
            },
        )
    }
}
