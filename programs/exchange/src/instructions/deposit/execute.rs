use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::accounts::{CheckRole, GetMarketMeta, RemoveDeposit},
    program::DataStore,
    states::{Deposit, MarketMeta},
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

impl<'info> AsMarket<'info> for ExecuteDeposit<'info> {
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

impl<'info> ExecuteDeposit<'info> {
    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        self.oracle.reload()?;
        let meta = get_market_meta(&self.data_store_program, self.market.to_account_info())?;
        let (long_amount, short_amount) = self.perform_swaps(&meta, remaining_accounts)?;
        msg!("{}, {}", long_amount, short_amount);
        self.perform_deposit(&meta, long_amount, short_amount)?;
        Ok(())
    }

    fn perform_swaps(
        &mut self,
        meta: &MarketMeta,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(u64, u64)> {
        let swap_params = &self.deposit.dynamic.swap_params;
        let long_len = swap_params.long_token_swap_path.len();
        let total_len =
            swap_params.long_token_swap_path.len() + swap_params.short_token_swap_path.len();

        // Expecting the `remaining_accounts` are of the of the following form:
        // [...long_path_markets, ...short_path_markets, ...long_path_mints, ...short_path_mints]
        require_gte!(
            remaining_accounts.len(),
            total_len * 2,
            ExchangeError::NotEnoughRemainingAccounts
        );

        // Markets.
        let long_swap_path = &remaining_accounts[0..long_len];
        let short_swap_path = &remaining_accounts[long_len..total_len];

        // Mints.
        let remaining_accounts = &remaining_accounts[total_len..];
        let long_swap_path_mints = &remaining_accounts[0..long_len];
        let short_swap_path_mints = &remaining_accounts[long_len..total_len];

        let long_amount = self.perform_swap(meta, true, long_swap_path, long_swap_path_mints)?;
        let short_amount =
            self.perform_swap(meta, false, short_swap_path, short_swap_path_mints)?;
        Ok((long_amount, short_amount))
    }

    fn perform_swap(
        &mut self,
        meta: &MarketMeta,
        is_long: bool,
        markets: &[AccountInfo<'info>],
        mints: &'info [AccountInfo<'info>],
    ) -> Result<u64> {
        let (token, final_token, mut amount, expected_mints) = if is_long {
            (
                self.deposit.fixed.tokens.initial_long_token,
                meta.long_token_mint,
                self.deposit.fixed.tokens.params.initial_long_token_amount,
                &self.deposit.dynamic.swap_params.long_token_swap_path,
            )
        } else {
            (
                self.deposit.fixed.tokens.initial_short_token,
                meta.short_token_mint,
                self.deposit.fixed.tokens.params.initial_short_token_amount,
                &self.deposit.dynamic.swap_params.short_token_swap_path,
            )
        };

        let Some(mut token_in) = token else {
            return Ok(0);
        };

        let mut flags = BTreeSet::default();
        for (idx, market) in markets.iter().enumerate() {
            require!(flags.insert(market.key), ExchangeError::InvalidSwapPath);
            let meta = get_market_meta(&self.data_store_program, market.clone())?;
            let mint = Account::<Mint>::try_from(&mints[idx])?;
            require_eq!(
                meta.market_token_mint,
                mint.key(),
                ExchangeError::InvalidSwapPath
            );
            require_eq!(
                mint.key(),
                expected_mints[idx],
                ExchangeError::InvalidSwapPath
            );
            require!(
                meta.long_token_mint != meta.short_token_mint,
                ExchangeError::InvalidSwapPath
            );
            let (is_token_in_long, token_out) = if token_in == meta.long_token_mint {
                (true, meta.short_token_mint)
            } else if token_in == meta.short_token_mint {
                (false, meta.long_token_mint)
            } else {
                return Err(ExchangeError::InvalidSwapPath.into());
            };
            let long_token_price = self
                .oracle
                .primary
                .get(&meta.long_token_mint)
                .ok_or(ExchangeError::MissingOraclePrice)?;
            let short_token_price = self
                .oracle
                .primary
                .get(&meta.short_token_mint)
                .ok_or(ExchangeError::MissingOraclePrice)?;
            let report = self
                .as_market(market.clone(), &mint)
                .swap(
                    is_token_in_long,
                    amount.into(),
                    long_token_price.max.to_unit_price(),
                    short_token_price.max.to_unit_price(),
                )
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(GmxCoreError::from)?;
            token_in = token_out;
            amount = (*report.token_out_amount())
                .try_into()
                .map_err(|_| ExchangeError::AmountOverflow)?;
            msg!("{:?}", report);
        }
        require_eq!(token_in, final_token, ExchangeError::InvalidSwapPath);
        Ok(amount)
    }

    fn perform_deposit(
        &mut self,
        meta: &MarketMeta,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let long_price = self
            .oracle
            .primary
            .get(&meta.long_token_mint)
            .ok_or(ExchangeError::MissingOraclePrice)?
            .max
            .to_unit_price();
        let short_price = self
            .oracle
            .primary
            .get(&meta.short_token_mint)
            .ok_or(ExchangeError::MissingOraclePrice)?
            .max
            .to_unit_price();
        self.as_market(self.market.to_account_info(), &self.market_token_mint)
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
        Ok(())
    }
}

fn get_market_meta<'info>(
    program: &Program<'info, DataStore>,
    market: AccountInfo<'info>,
) -> Result<MarketMeta> {
    let ctx = CpiContext::new(program.to_account_info(), GetMarketMeta { market });
    Ok(data_store::cpi::get_market_meta(ctx)?.get())
}
