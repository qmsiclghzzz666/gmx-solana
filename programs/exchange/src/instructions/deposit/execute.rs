use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::{
        self,
        accounts::{CheckRole, RemoveDeposit},
    },
    program::DataStore,
    states::Deposit,
    utils::Authentication,
};
use oracle::{
    program::Oracle,
    utils::{Chainlink, WithOracle, WithOracleExt},
};

use crate::ExchangeError;

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    execution_fee: u64,
) -> Result<()> {
    let deposit = &ctx.accounts.deposit;
    let refund = deposit
        .get_lamports()
        .checked_sub(super::MAX_DEPOSIT_EXECUTION_FEE.min(execution_fee))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    ctx.accounts.with_oracle_prices(
        deposit.dynamic.tokens_with_feed.tokens.clone(),
        ctx.remaining_accounts,
        |accounts, remaining_accounts| accounts.execute(remaining_accounts),
    )?;
    data_store::cpi::remove_deposit(ctx.accounts.remove_deposit_ctx(), refund)
}

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: used and checked by CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub oracle_program: Program<'info, Oracle>,
    pub chainlink_program: Program<'info, Chainlink>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub oracle: Account<'info, data_store::states::Oracle>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = receiver.key() == deposit.fixed.receivers.receiver)]
    pub receiver: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.key() == deposit.fixed.market)]
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == deposit.fixed.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> ExecuteDeposit<'info> {
    fn remove_deposit_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveDeposit {
                authority: self.authority.to_account_info(),
                only_controller: self.only_order_keeper.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn execute_deposit_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, cpi::accounts::ExecuteDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            cpi::accounts::ExecuteDeposit {
                authority: self.authority.to_account_info(),
                only_order_keeper: self.only_order_keeper.to_account_info(),
                store: self.store.to_account_info(),
                oracle: self.oracle.to_account_info(),
                deposit: self.deposit.to_account_info(),
                market: self.market.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                receiver: self.receiver.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

impl<'info> Authentication<'info> for ExecuteDeposit<'info> {
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

impl<'info> WithOracle<'info> for ExecuteDeposit<'info> {
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

impl<'info> ExecuteDeposit<'info> {
    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        // self.oracle.reload()?;
        cpi::execute_deposit(
            self.execute_deposit_ctx()
                .with_remaining_accounts(remaining_accounts.to_vec()),
        )?;
        Ok(())
    }
}
