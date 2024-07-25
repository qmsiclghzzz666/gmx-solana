use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::accounts::RemovePosition,
    program::GmsolStore,
    states::{
        order::{OrderKind, TransferOut},
        Oracle, Order, PriceProvider,
    },
    utils::{Authentication, WithOracle, WithOracleExt, WithStore},
};

use crate::{
    utils::{must_be_uninitialized, ControllerSeeds},
    ExchangeError,
};

use super::utils::{CancelOrderUtil, TransferOutUtils};

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
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
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == order.fixed.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = order.fixed.senders.initial_collateral_token_account == initial_collateral_token_account.as_ref().map(|a| a.key()),
    )]
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
    /// CHECK: check by CPI.
    #[account(mut)]
    pub long_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub short_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub long_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub short_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_long_token_account_for_user: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_short_token_account_for_user: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_pnl_token_account_for_holding: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_collateral_token_account: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_collateral_token_vault: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI and cancel utils.
    #[account(mut)]
    pub initial_market: Option<UncheckedAccount<'info>>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
    pub token_program: Program<'info, Token>,
    pub price_provider: Interface<'info, PriceProvider>,
    pub system_program: Program<'info, System>,
}

/// Execute an order.
pub fn execute_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
    recent_timestamp: i64,
    execution_fee: u64,
    cancel_on_execution_error: bool,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);
    let is_executable = ctx.accounts.is_executable()?;

    let is_executed = if is_executable {
        // TODO: validate the pre-condition of transferring out before execution.
        let (
            should_remove_position,
            transfer_out,
            final_output_market,
            final_secondary_output_market,
        ) = ctx.accounts.execute(
            recent_timestamp,
            cancel_on_execution_error,
            &controller,
            ctx.remaining_accounts,
        )?;
        ctx.accounts.process_transfer_out(
            &controller,
            &transfer_out,
            final_output_market,
            final_secondary_output_market,
        )?;
        if should_remove_position {
            // Refund all lamports.
            let refund = ctx.accounts.position()?.get_lamports();
            gmsol_store::cpi::remove_position(
                ctx.accounts
                    .remove_position_ctx()?
                    .with_signer(&[&controller.as_seeds()]),
                refund,
            )?;
        }
        transfer_out.executed
    } else {
        false
    };

    let reason = if is_executed {
        "executed"
    } else {
        "execution failed"
    };
    ctx.accounts.cancel_util(reason).execute(
        ctx.accounts.authority.to_account_info(),
        &controller,
        execution_fee,
    )?;
    Ok(())
}

impl<'info> WithStore<'info> for ExecuteOrder<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for ExecuteOrder<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> WithOracle<'info> for ExecuteOrder<'info> {
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

impl<'info> ExecuteOrder<'info> {
    fn is_executable(&self) -> Result<bool> {
        match self.order.fixed.params.kind {
            OrderKind::MarketIncrease | OrderKind::MarketDecrease | OrderKind::Liquidation => {
                let position = self
                    .position
                    .as_ref()
                    .ok_or(error!(ExchangeError::PositionNotProvided))?;
                // The order is not executable if the position have not been initialized.
                Ok(!must_be_uninitialized(position))
            }
            OrderKind::MarketSwap => Ok(true),
            _ => {
                err!(ExchangeError::UnsupportedOrderKind)
            }
        }
    }

    /// Execute the order and return the result.
    ///
    /// Return `(should_remove_position, transfer_out, final_output_market, final_secondary_output_market)`.
    // We use `#[inline(never)]` here to force the compiler to create a new stack frame for us.
    #[inline(never)]
    fn execute(
        &mut self,
        recent_timestamp: i64,
        cancel_on_execution_error: bool,
        controller: &ControllerSeeds,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(
        bool,
        Box<TransferOut>,
        AccountInfo<'info>,
        AccountInfo<'info>,
    )> {
        self.with_oracle_prices(
            self.order.prices.tokens.clone(),
            remaining_accounts,
            &controller.as_seeds(),
            |accounts, remaining_accounts| {
                let store = accounts.store.key;
                let swap = &accounts.order.swap;
                let final_output_market = swap
                    .find_last_market(store, true, remaining_accounts)
                    .unwrap_or(accounts.market.to_account_info());
                let final_secondary_output_market = swap
                    .find_last_market(store, false, remaining_accounts)
                    .unwrap_or(accounts.market.to_account_info());
                let (should_remove_position, transfer_out) = gmsol_store::cpi::execute_order(
                    accounts
                        .execute_order_ctx()
                        .with_signer(&[&controller.as_seeds()])
                        .with_remaining_accounts(remaining_accounts.to_vec()),
                    recent_timestamp,
                    !cancel_on_execution_error,
                )?
                .get();
                accounts.order.reload()?;
                Ok((
                    should_remove_position,
                    transfer_out,
                    final_output_market,
                    final_secondary_output_market,
                ))
            },
        )
    }

    fn cancel_util<'a>(&'a self, reason: &'a str) -> CancelOrderUtil<'a, 'info> {
        CancelOrderUtil {
            event_authority: self.event_authority.to_account_info(),
            data_store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            controller: self.controller.to_account_info(),
            store: self.store.to_account_info(),
            user: self.user.to_account_info(),
            order: &self.order,
            initial_market: self.initial_market.as_ref().map(|a| a.to_account_info()),
            initial_collateral_token_account: self
                .initial_collateral_token_account
                .as_ref()
                .map(|a| a.to_account_info()),
            initial_collateral_token_vault: self
                .initial_collateral_token_vault
                .as_ref()
                .map(|a| a.to_account_info()),
            reason,
        }
    }

    fn execute_order_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, gmsol_store::cpi::accounts::ExecuteOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            gmsol_store::cpi::accounts::ExecuteOrder {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
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
                long_token_vault: self.long_token_vault.to_account_info(),
                short_token_vault: self.short_token_vault.to_account_info(),
                long_token_account: self.long_token_account.to_account_info(),
                short_token_account: self.short_token_account.to_account_info(),
                token_program: self.token_program.to_account_info(),
                claimable_long_token_account_for_user: self
                    .claimable_long_token_account_for_user
                    .as_ref()
                    .map(|a| a.to_account_info()),
                claimable_short_token_account_for_user: self
                    .claimable_short_token_account_for_user
                    .as_ref()
                    .map(|a| a.to_account_info()),
                claimable_pnl_token_account_for_holding: self
                    .claimable_pnl_token_account_for_holding
                    .as_ref()
                    .map(|a| a.to_account_info()),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }

    fn position(&self) -> Result<&UncheckedAccount<'info>> {
        let Some(position) = self.position.as_ref() else {
            return err!(ExchangeError::PositionNotProvided);
        };
        Ok(position)
    }

    fn remove_position_ctx(&self) -> Result<CpiContext<'_, '_, '_, 'info, RemovePosition<'info>>> {
        Ok(CpiContext::new(
            self.data_store_program.to_account_info(),
            RemovePosition {
                payer: self.authority.to_account_info(),
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                position: self.position()?.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        ))
    }

    fn transfer_out_utils(
        &self,
        final_output_market: AccountInfo<'info>,
        final_secondary_output_market: AccountInfo<'info>,
    ) -> TransferOutUtils<'info> {
        TransferOutUtils {
            store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            controller: self.controller.to_account_info(),
            market: self.market.to_account_info(),
            store: self.store.to_account_info(),
            long_token_vault: self.long_token_vault.to_account_info(),
            long_token_account: self.long_token_account.to_account_info(),
            short_token_vault: self.short_token_vault.to_account_info(),
            short_token_account: self.short_token_account.to_account_info(),
            final_output_token_account: self
                .final_output_token_account
                .as_ref()
                .map(|a| a.to_account_info()),
            final_output_token_vault: self
                .final_output_token_vault
                .as_ref()
                .map(|a| a.to_account_info()),
            final_output_market,
            secondary_output_token_account: self
                .secondary_output_token_account
                .as_ref()
                .map(|a| a.to_account_info()),
            secondary_output_token_vault: self
                .secondary_output_token_vault
                .as_ref()
                .map(|a| a.to_account_info()),
            final_secondary_output_market,
            claimable_long_token_account_for_user: self
                .claimable_long_token_account_for_user
                .as_ref()
                .map(|a| a.to_account_info()),
            claimable_short_token_account_for_user: self
                .claimable_short_token_account_for_user
                .as_ref()
                .map(|a| a.to_account_info()),
            claimable_pnl_token_account_for_holding: self
                .claimable_pnl_token_account_for_holding
                .as_ref()
                .map(|a| a.to_account_info()),
        }
    }

    fn process_transfer_out(
        &self,
        controller: &ControllerSeeds,
        transfer_out: &TransferOut,
        final_output_market: AccountInfo<'info>,
        final_secondary_output_market: AccountInfo<'info>,
    ) -> Result<()> {
        let utils = self.transfer_out_utils(final_output_market, final_secondary_output_market);
        // CHECK: `transfer_out` are validated during the execution.
        utils.unchecked_process(controller, transfer_out)?;
        Ok(())
    }
}
