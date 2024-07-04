use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi,
    program::GmsolStore,
    states::{Deposit, PriceProvider},
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::{utils::ControllerSeeds, ExchangeError};

use super::utils::{CancelDepositUtils, TransferIn};

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
    pub price_provider: Interface<'info, PriceProvider>,
    #[account(mut)]
    pub oracle: Account<'info, gmsol_store::states::Oracle>,
    #[account(
        mut,
        constraint = initial_long_token_account.as_ref().map(|a| a.key()) == deposit.fixed.senders.initial_long_token_account,
        constraint = initial_short_token_account.as_ref().map(|a| a.key()) == deposit.fixed.senders.initial_short_token_account,
    )]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = receiver.key() == deposit.fixed.receivers.receiver)]
    pub receiver: Account<'info, TokenAccount>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    #[account(mut, constraint = market.key() == deposit.fixed.market)]
    pub market: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == deposit.fixed.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    /// CHECK: check by `try_removable` method.
    #[account(mut)]
    pub initial_long_token_account: Option<UncheckedAccount<'info>>,
    /// CHECK: check by `try_removable` method.
    #[account(mut)]
    pub initial_short_token_account: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_long_token_vault: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_short_token_vault: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI and cancel utils.
    #[account(mut)]
    pub initial_long_market: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI and cancel utils.
    #[account(mut)]
    pub initial_short_market: Option<UncheckedAccount<'info>>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    execution_fee: u64,
    cancel_on_execution_error: bool,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    let mut reason = "execution failed";
    // Try to execute the deposit.
    if ctx.accounts.is_executable()? {
        let deposit = &ctx.accounts.deposit;
        let executed = ctx.accounts.with_oracle_prices(
            deposit.dynamic.tokens_with_feed.tokens.clone(),
            ctx.remaining_accounts,
            &controller.as_seeds(),
            |accounts, remaining_accounts| {
                accounts.execute(&controller, remaining_accounts, cancel_on_execution_error)
            },
        )?;
        if executed {
            reason = "executed";
        }
    }

    ctx.accounts
        .try_remove(&controller, execution_fee, reason)?;
    Ok(())
}

impl<'info> ExecuteDeposit<'info> {
    fn is_executable(&self) -> Result<bool> {
        Ok(!crate::utils::must_be_uninitialized(&self.receiver))
    }

    fn initial_transfer_in(&self, is_long: bool) -> Option<TransferIn<'info>> {
        if is_long {
            TransferIn::new(
                self.initial_long_token_account.as_ref(),
                self.initial_long_token_vault.as_ref(),
                self.initial_long_market.as_ref(),
            )
        } else {
            TransferIn::new(
                self.initial_short_token_account.as_ref(),
                self.initial_short_token_vault.as_ref(),
                self.initial_short_market.as_ref(),
            )
        }
    }

    fn try_removable(&self) -> Result<bool> {
        if self.deposit.fixed.tokens.params.initial_long_token_amount != 0
            && self
                .initial_transfer_in(true)
                .map(|t| t.is_from_account_not_initialized())
                .unwrap_or(true)
        {
            msg!("invalid `transfer_in` accounts for long");
            return Ok(false);
        }
        if self.deposit.fixed.tokens.params.initial_short_token_amount != 0
            && self
                .initial_transfer_in(false)
                .map(|t| t.is_from_account_not_initialized())
                .unwrap_or(true)
        {
            msg!("invalid `transfer_in` accounts for short");
            return Ok(false);
        }
        Ok(true)
    }

    fn cancel_utils<'a>(&'a self, reason: &'a str) -> CancelDepositUtils<'a, 'info> {
        CancelDepositUtils {
            event_authority: self.event_authority.to_account_info(),
            data_store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            store: self.store.to_account_info(),
            controller: self.controller.to_account_info(),
            user: self.user.to_account_info(),
            deposit: &self.deposit,
            initial_long_token_transfer: self.initial_transfer_in(true),
            initial_short_token_transfer: self.initial_transfer_in(false),
            reason,
        }
    }

    fn try_remove(
        &self,
        controller: &ControllerSeeds,
        execution_fee: u64,
        reason: &str,
    ) -> Result<()> {
        if self.try_removable()? {
            self.cancel_utils(reason).execute(
                self.authority.to_account_info(),
                controller,
                execution_fee,
            )?;
        } else {
            // TODO: emit an event.
            msg!("[Unable to cancel] the deposit cannot be removed automatically, should be removed by the user manually");
        }
        Ok(())
    }

    fn execute_deposit_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, cpi::accounts::ExecuteDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            cpi::accounts::ExecuteDeposit {
                authority: self.controller.to_account_info(),
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

    fn controller(&self) -> AccountInfo<'info> {
        self.controller.to_account_info()
    }
}

impl<'info> ExecuteDeposit<'info> {
    fn execute(
        &mut self,
        controller_seeds: &ControllerSeeds,
        remaining_accounts: &'info [AccountInfo<'info>],
        cancel_on_execution_error: bool,
    ) -> Result<bool> {
        let executed = cpi::execute_deposit(
            self.execute_deposit_ctx()
                .with_signer(&[&controller_seeds.as_seeds()])
                .with_remaining_accounts(remaining_accounts.to_vec()),
            !cancel_on_execution_error,
        )?
        .get();
        self.deposit.reload()?;
        Ok(executed)
    }
}
