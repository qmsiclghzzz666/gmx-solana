use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::CheckRole, program::DataStore, states::Deposit, utils::Authentication,
};
use oracle::{
    program::Oracle,
    utils::{Chainlink, WithOracle, WithOracleExt},
};

use crate::ExchangeError;

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
) -> Result<()> {
    let deposit = &ctx.accounts.deposit;
    let long_token = deposit.tokens.initial_long_token;
    let short_token = deposit.tokens.initial_short_token;
    let remaining_accounts = ctx.remaining_accounts.to_vec();
    ctx.accounts.with_oracle_prices(
        vec![long_token, short_token],
        remaining_accounts,
        |accounts| {
            let oracle = &mut accounts.oracle;
            oracle.reload()?;
            let long_price = oracle.primary.get(&long_token).unwrap().max.to_unit_price();
            let short_price = oracle
                .primary
                .get(&short_token)
                .unwrap()
                .max
                .to_unit_price();
            msg!(&long_price.to_string());
            msg!(&short_price.to_string());
            Ok(())
        },
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    pub authority: Signer<'info>,
    /// CHECK: used and checked by CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub oracle_program: Program<'info, Oracle>,
    pub chainlink_program: Program<'info, Chainlink>,
    #[account(mut)]
    pub oracle: Account<'info, data_store::states::Oracle>,
    /// CHECK: used and checked by CPI.
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
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
}
