use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use data_store::{
    program::DataStore,
    states::{Chainlink, Oracle, Order},
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::ExchangeError;

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: used and checked by CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == order.fixed.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub order: Account<'info, Order>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub position: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub final_output_token_vault: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub secondary_output_token_vault: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub final_output_token_account: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub secondary_output_token_account: Option<UncheckedAccount<'info>>,
    pub data_store_program: Program<'info, DataStore>,
    pub token_program: Program<'info, Token>,
    pub chainlink_program: Program<'info, Chainlink>,
    pub system_program: Program<'info, System>,
}

/// Execute an order.
pub fn execute_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
    execution_fee: u64,
) -> Result<()> {
    let order = &ctx.accounts.order;
    let _refund = order
        .get_lamports()
        .checked_sub(super::MAX_ORDER_EXECUTION_FEE.min(execution_fee))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    let _should_remove_position = ctx.accounts.with_oracle_prices(
        order.prices.tokens.clone(),
        ctx.remaining_accounts,
        |accounts, remaining_accounts| {
            let should_remove_position = data_store::cpi::execute_order(
                accounts
                    .execute_order_ctx()
                    .with_remaining_accounts(remaining_accounts.to_vec()),
            )?
            .get();
            Ok(should_remove_position)
        },
    )?;
    // TODO: remove order.
    Ok(())
}

impl<'info> Authentication<'info> for ExecuteOrder<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }

    fn data_store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }

    fn roles(&self) -> AccountInfo<'info> {
        self.only_order_keeper.to_account_info()
    }
}

impl<'info> WithOracle<'info> for ExecuteOrder<'info> {
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

impl<'info> ExecuteOrder<'info> {
    fn execute_order_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, data_store::cpi::accounts::ExecuteOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            data_store::cpi::accounts::ExecuteOrder {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_order_keeper: self.only_order_keeper.to_account_info(),
                oracle: self.oracle.to_account_info(),
                order: self.order.to_account_info(),
                market: self.market.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                position: self.position.as_ref().map(|a| a.to_account_info()),
                final_output_token_vault: self
                    .final_output_token_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                secondary_output_token_vault: self
                    .secondary_output_token_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                final_output_token_account: self
                    .final_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                secondary_output_token_account: self
                    .secondary_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
