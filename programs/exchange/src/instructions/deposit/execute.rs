use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::accounts::{CheckRole, RemoveDeposit},
    program::DataStore,
    states::Deposit,
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
    // TODO: perform the swaps.
    let long_token = deposit.tokens.params.initial_long_token;
    let short_token = deposit.tokens.params.initial_short_token;
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
            let (long_amount, short_amount) = (
                accounts.deposit.tokens.params.initial_long_token_amount,
                accounts.deposit.tokens.params.initial_short_token_amount,
            );
            let report = accounts
                .as_market()
                .deposit(
                    long_amount.into(),
                    short_amount.into(),
                    long_price,
                    short_price,
                )
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(|err| {
                    msg!(&err.to_string());
                    GmxCoreError::from(err)
                })?;
            msg!("{:?}", report);
            Ok(())
        },
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
    /// CHECK: used and checked by CPI.
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = receiver.key() == deposit.receivers.receiver)]
    pub receiver: Account<'info, TokenAccount>,
    #[account(mut, constraint = market.key() == deposit.market)]
    /// CHECK: only used to invoke CPI and should be checked by it.
    pub market: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == deposit.tokens.market_token)]
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

impl<'info> AsMarket<'info> for ExecuteDeposit<'info> {
    fn market(&self) -> AccountInfo<'info> {
        self.market.to_account_info()
    }

    fn market_token(&self) -> &Account<'info, Mint> {
        &self.market_token_mint
    }

    fn receiver(&self) -> Option<&Account<'info, TokenAccount>> {
        Some(&self.receiver)
    }

    fn withdrawal_vault(&self) -> Option<&Account<'info, TokenAccount>> {
        None
    }

    fn token_program(&self) -> AccountInfo<'info> {
        self.token_program.to_account_info()
    }
}
