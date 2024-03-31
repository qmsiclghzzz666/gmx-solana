use std::collections::BTreeSet;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount};
use data_store::{
    cpi::accounts::{CheckRole, GetMarketMeta, GetTokenConfig, InitializeWithdrawal},
    program::DataStore,
    states::{withdrawal::TokenParams, NonceBytes},
    utils::Authentication,
};

use crate::ExchangeError;

/// Create Withdrawal Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateWithdrawalParams {
    pub market_token_amount: u64,
    pub execution_fee: u64,
    pub ui_fee_receiver: Pubkey,
    pub tokens: TokenParams,
}

#[derive(Accounts)]
pub struct CreateWithdrawal<'info> {
    pub authority: Signer<'info>,
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
    pub market_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = market_token.mint,
    )]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    pub final_short_token_receiver: Account<'info, TokenAccount>,
}

/// Create withdrawal.
pub fn create_withdrawal(
    ctx: Context<CreateWithdrawal>,
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
    let market_meta = cpi::get_market_meta(ctx.accounts.get_market_meta_ctx())?.get();
    require_eq!(
        ctx.accounts.market_token.mint,
        market_meta.market_token_mint,
        ExchangeError::MismatchedMarketTokenMint
    );
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

    // Transfer the market tokens to the vault.
    token::transfer(
        ctx.accounts.token_transfer_ctx(),
        params.market_token_amount,
    )?;

    cpi::initialize_withdrawal(
        ctx.accounts.initialize_withdrawal_ctx(),
        nonce,
        params.tokens,
        tokens_with_feed,
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
    Ok(())
}

impl<'info> Authentication<'info> for CreateWithdrawal<'info> {
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

impl<'info> CreateWithdrawal<'info> {
    fn get_token_config_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetTokenConfig<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetTokenConfig {
                map: self.token_config_map.to_account_info(),
            },
        )
    }

    fn get_market_meta_ctx(&self) -> CpiContext<'_, '_, '_, 'info, GetMarketMeta<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            GetMarketMeta {
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
                market: self.market.to_account_info(),
                final_long_token_receiver: self.final_long_token_receiver.to_account_info(),
                final_short_token_receiver: self.final_short_token_receiver.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn token_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.market_token.to_account_info(),
                to: self.market_token_withdrawal_vault.to_account_info(),
                authority: self.payer.to_account_info(),
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
