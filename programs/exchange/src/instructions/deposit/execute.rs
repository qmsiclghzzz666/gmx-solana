use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::{self, accounts::RemoveDeposit},
    program::DataStore,
    states::{Deposit, PriceProvider},
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::{utils::ControllerSeeds, ExchangeError};

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    execution_fee: u64,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);
    let deposit = &ctx.accounts.deposit;
    let refund = deposit
        .get_lamports()
        .checked_sub(super::MAX_DEPOSIT_EXECUTION_FEE.min(execution_fee))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    ctx.accounts.with_oracle_prices(
        deposit.dynamic.tokens_with_feed.tokens.clone(),
        ctx.remaining_accounts,
        |accounts, remaining_accounts| accounts.execute(&controller, remaining_accounts),
    )?;
    data_store::cpi::remove_deposit(
        ctx.accounts
            .remove_deposit_ctx()
            .with_signer(&[&controller.as_seeds()]),
        refund,
    )
}

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub token_map: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub price_provider: Interface<'info, PriceProvider>,
    pub token_program: Program<'info, Token>,
    /// CHECK: only use and check by CPI.
    pub config: UncheckedAccount<'info>,
    #[account(mut)]
    pub oracle: Account<'info, data_store::states::Oracle>,
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
                payer: self.authority.to_account_info(),
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                initial_long_token: None,
                initial_short_token: None,
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn execute_deposit_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, cpi::accounts::ExecuteDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            cpi::accounts::ExecuteDeposit {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                config: self.config.to_account_info(),
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
}

impl<'info> WithOracle<'info> for ExecuteDeposit<'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_map(&self) -> AccountInfo<'info> {
        self.token_map.to_account_info()
    }

    fn config(&self) -> AccountInfo<'info> {
        self.config.to_account_info()
    }
}

impl<'info> ExecuteDeposit<'info> {
    fn execute(
        &mut self,
        controller_seeds: &ControllerSeeds,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        cpi::execute_deposit(
            self.execute_deposit_ctx()
                .with_signer(&[&controller_seeds.as_seeds()])
                .with_remaining_accounts(remaining_accounts.to_vec()),
        )?;
        Ok(())
    }
}
