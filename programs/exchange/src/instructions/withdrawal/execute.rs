use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::accounts::{CheckRole, RemoveWithdrawal},
    program::DataStore,
    states::Withdrawal,
    utils::Authentication,
};
use oracle::{
    program::Oracle,
    utils::{Chainlink, WithOracle, WithOracleExt},
};

use crate::{utils::market::AsMarket, ExchangeError};

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
    /// CHECK: used and checked by CPI.
    #[account(mut)]
    pub withdrawal: Account<'info, Withdrawal>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == withdrawal.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
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
    // TODO: fetch market's long short tokens.
    let long_token = withdrawal.tokens.final_long_token;
    let short_token = withdrawal.tokens.final_short_token;
    let remaing_accounts = ctx.remaining_accounts.to_vec();
    ctx.accounts.with_oracle_prices(
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
            msg!("{}, {}", long_token_price, short_token_price);
            Ok(())
        },
    )?;
    // TODO: perform the swaps.
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

    fn token_program(&self) -> AccountInfo<'info> {
        self.token_program.to_account_info()
    }
}
