use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::accounts::{CheckRole, GetMarketMeta, MarketVaultTransferOut, RemoveWithdrawal},
    program::DataStore,
    states::Withdrawal,
    utils::Authentication,
};
use gmx_core::MarketExt;
use oracle::{
    program::Oracle,
    utils::{Chainlink, WithOracle, WithOracleExt},
};

use crate::{
    utils::market::{AsMarket, GmxCoreError},
    ExchangeError,
};

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub oracle_program: Program<'info, Oracle>,
    pub chainlink_program: Program<'info, Chainlink>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub oracle: Account<'info, data_store::states::Oracle>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    ///
    /// ## Notes
    /// - `user` is checked on the removal CPI of the withdrawal.
    #[account(
        mut,
        constraint = withdrawal.tokens.market_token == market_token_mint.key() @ ExchangeError::InvalidWIthdrawalToExecute,
        constraint = withdrawal.receivers.final_long_token_receiver == final_long_token_receiver.key() @ ExchangeError::InvalidWIthdrawalToExecute,
        constraint = withdrawal.receivers.final_short_token_receiver == final_short_token_receiver.key() @ ExchangeError::InvalidWIthdrawalToExecute,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == withdrawal.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = market_token_mint)]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_vault: Account<'info, TokenAccount>,
}

/// Execute the withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    execution_fee: u64,
) -> Result<()> {
    let withdrawal = &ctx.accounts.withdrawal;
    let refund = withdrawal
        .get_lamports()
        .checked_sub(execution_fee.min(super::MAX_WITHDRAWAL_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    let market_token_amount = withdrawal.tokens.market_token_amount;
    let min_long_token_amount = withdrawal.tokens.params.min_long_token_amount;
    let min_short_token_amount = withdrawal.tokens.params.min_short_token_amount;
    let meta = data_store::cpi::get_market_meta(ctx.accounts.get_market_meta_ctx())?.get();
    let long_token = meta.long_token_mint;
    let short_token = meta.short_token_mint;
    let remaing_accounts = ctx.remaining_accounts.to_vec();
    let report = ctx.accounts.with_oracle_prices(
        vec![long_token, short_token],
        remaing_accounts,
        |accounts| {
            let oracle = &mut accounts.oracle;
            oracle.reload()?;
            let long_token_price = oracle.primary.get(&long_token).unwrap().max.to_unit_price();
            let short_token_price = oracle
                .primary
                .get(&short_token)
                .unwrap()
                .max
                .to_unit_price();
            let report = accounts
                .as_market()
                .withdraw(
                    market_token_amount.into(),
                    long_token_price,
                    short_token_price,
                )
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(|err| {
                    msg!(&err.to_string());
                    GmxCoreError::from(err)
                })?;
            Ok(report)
        },
    )?;
    msg!("{:?}", report);
    // TODO: perform the swaps.
    // For now we are assuming that final tokens are the same as pool tokens.
    let final_long_token_amount: u64 = (*report.long_token_output())
        .try_into()
        .map_err(|_| ExchangeError::InvalidOutputAmount)?;
    let final_short_token_amount: u64 = (*report.short_token_output())
        .try_into()
        .map_err(|_| ExchangeError::InvalidOutputAmount)?;
    require_gte!(
        final_long_token_amount,
        min_long_token_amount,
        ExchangeError::OutputAmountTooSmall
    );
    require_gte!(
        final_short_token_amount,
        min_short_token_amount,
        ExchangeError::OutputAmountTooSmall
    );
    data_store::cpi::market_vault_transfer_out(
        ctx.accounts.market_vault_transfer_out_ctx(true),
        final_long_token_amount,
    )?;
    data_store::cpi::market_vault_transfer_out(
        ctx.accounts.market_vault_transfer_out_ctx(false),
        final_short_token_amount,
    )?;
    data_store::cpi::remove_withdrawal(ctx.accounts.remove_withdrawal_ctx(), refund)?;
    Ok(())
}

impl<'info> Authentication<'info> for ExecuteWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            CheckRole {
                store: self.store.to_account_info(),
                roles: self.only_order_keeper.to_account_info(),
            },
        )
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> WithOracle<'info> for ExecuteWithdrawal<'info> {
    fn oracle_program(&self) -> AccountInfo<'info> {
        self.oracle_program.to_account_info()
    }

    fn chainlink_program(&self) -> AccountInfo<'info> {
        self.chainlink_program.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_config_map(&self) -> AccountInfo<'info> {
        self.token_config_map.to_account_info()
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn remove_withdrawal_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveWithdrawal {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_order_keeper.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
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

    fn market_vault_transfer_out_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, MarketVaultTransferOut<'info>> {
        let (market_vault, to) = if is_long_token {
            (
                self.final_long_token_vault.to_account_info(),
                self.final_long_token_receiver.to_account_info(),
            )
        } else {
            (
                self.final_short_token_vault.to_account_info(),
                self.final_short_token_receiver.to_account_info(),
            )
        };
        CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketVaultTransferOut {
                authority: self.authority.to_account_info(),
                only_controller: self.only_order_keeper.to_account_info(),
                store: self.store.to_account_info(),
                market_vault,
                to,
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

impl<'info> AsMarket<'info> for ExecuteWithdrawal<'info> {
    fn market(&self) -> AccountInfo<'info> {
        self.market.to_account_info()
    }

    fn market_token(&self) -> &Account<'info, Mint> {
        &self.market_token_mint
    }

    fn receiver(&self) -> Option<&Account<'info, TokenAccount>> {
        None
    }

    fn withdrawal_vault(&self) -> Option<&Account<'info, TokenAccount>> {
        Some(&self.market_token_withdrawal_vault)
    }

    fn token_program(&self) -> AccountInfo<'info> {
        self.token_program.to_account_info()
    }
}
