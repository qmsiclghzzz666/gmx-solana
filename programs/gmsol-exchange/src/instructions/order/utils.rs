use anchor_lang::prelude::*;

use gmsol_store::{
    cpi::accounts::{MarketTransferOut, RemoveOrder},
    states::{order::TransferOut, Order},
};

use crate::{
    utils::{market::get_market_token_mint, ControllerSeeds},
    ExchangeError,
};

pub(crate) struct CancelOrderUtil<'a, 'info> {
    pub(super) data_store_program: AccountInfo<'info>,
    pub(super) event_authority: AccountInfo<'info>,
    pub(super) token_program: AccountInfo<'info>,
    pub(super) system_program: AccountInfo<'info>,
    pub(super) controller: AccountInfo<'info>,
    pub(super) store: AccountInfo<'info>,
    pub(super) user: AccountInfo<'info>,
    pub(super) order: &'a Account<'info, Order>,
    pub(super) initial_market: Option<AccountInfo<'info>>,
    pub(super) initial_collateral_token_account: Option<AccountInfo<'info>>,
    pub(super) initial_collateral_token_vault: Option<AccountInfo<'info>>,
    pub(super) reason: &'a str,
}

impl<'a, 'info> CancelOrderUtil<'a, 'info> {
    pub(crate) fn execute(
        self,
        payer: AccountInfo<'info>,
        controller: &ControllerSeeds,
        execution_fee: u64,
    ) -> Result<()> {
        let amount_to_cancel = if self.order.fixed.params.need_to_transfer_in() {
            self.order.fixed.params.initial_collateral_delta_amount
        } else {
            0
        };
        if amount_to_cancel != 0 {
            gmsol_store::cpi::market_transfer_out(
                self.market_transfer_out_ctx()?
                    .with_signer(&[&controller.as_seeds()]),
                amount_to_cancel,
            )?;
        }
        let refund = self
            .order
            .get_lamports()
            .checked_sub(execution_fee.min(crate::MAX_ORDER_EXECUTION_FEE))
            .ok_or(ExchangeError::NotEnoughExecutionFee)?;
        gmsol_store::cpi::remove_order(
            self.remove_order_ctx(payer)
                .with_signer(&[&controller.as_seeds()]),
            refund,
            self.reason.to_string(),
        )?;
        Ok(())
    }

    fn market_transfer_out_ctx(
        &self,
    ) -> Result<CpiContext<'_, '_, '_, 'info, MarketTransferOut<'info>>> {
        let (market, to, vault) = match (
            &self.initial_market,
            &self.initial_collateral_token_account,
            &self.initial_collateral_token_vault,
        ) {
            (Some(market), Some(account), Some(vault)) => (
                market.to_account_info(),
                account.to_account_info(),
                vault.to_account_info(),
            ),
            _ => {
                return err!(ExchangeError::InvalidArgument);
            }
        };
        let market_token = get_market_token_mint(&self.data_store_program, &market)?;
        let expected_market_token = self
            .order
            .swap
            .first_market_token(true)
            .unwrap_or(&self.order.fixed.tokens.market_token);
        require_eq!(
            market_token,
            *expected_market_token,
            ExchangeError::InvalidArgument
        );
        let ctx = CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketTransferOut {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market,
                to,
                vault,
                token_program: self.token_program.to_account_info(),
            },
        );
        Ok(ctx)
    }

    fn remove_order_ctx(
        &self,
        payer: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, RemoveOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveOrder {
                payer,
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                order: self.order.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }
}

pub(crate) struct TransferOutUtils<'info> {
    pub(super) store_program: AccountInfo<'info>,
    pub(super) token_program: AccountInfo<'info>,
    pub(super) controller: AccountInfo<'info>,
    pub(super) market: AccountInfo<'info>,
    pub(super) store: AccountInfo<'info>,
    pub(super) long_token_vault: AccountInfo<'info>,
    pub(super) long_token_account: AccountInfo<'info>,
    pub(super) short_token_vault: AccountInfo<'info>,
    pub(super) short_token_account: AccountInfo<'info>,
    pub(super) final_output_token_account: Option<AccountInfo<'info>>,
    pub(super) final_output_token_vault: Option<AccountInfo<'info>>,
    pub(super) final_output_market: AccountInfo<'info>,
    pub(super) secondary_output_token_account: Option<AccountInfo<'info>>,
    pub(super) secondary_output_token_vault: Option<AccountInfo<'info>>,
    pub(super) final_secondary_output_market: AccountInfo<'info>,
    pub(super) claimable_long_token_account_for_user: Option<AccountInfo<'info>>,
    pub(super) claimable_short_token_account_for_user: Option<AccountInfo<'info>>,
    pub(super) claimable_pnl_token_account_for_holding: Option<AccountInfo<'info>>,
}

impl<'info> TransferOutUtils<'info> {
    fn market_transfer_out_ctx(
        &self,
        market: AccountInfo<'info>,
        vault: AccountInfo<'info>,
        to: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, MarketTransferOut<'info>> {
        CpiContext::new(
            self.store_program.to_account_info(),
            MarketTransferOut {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market,
                to,
                vault,
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn market_transfer_out(
        &self,
        controller: &ControllerSeeds,
        market: Option<AccountInfo<'info>>,
        vault: Option<AccountInfo<'info>>,
        to: Option<AccountInfo<'info>>,
        amount: u64,
    ) -> Result<()> {
        gmsol_store::cpi::market_transfer_out(
            self.market_transfer_out_ctx(
                market.ok_or(error!(ExchangeError::InvalidArgument))?,
                vault.ok_or(error!(ExchangeError::InvalidArgument))?,
                to.ok_or(error!(ExchangeError::InvalidArgument))?,
            )
            .with_signer(&[&controller.as_seeds()]),
            amount,
        )?;
        Ok(())
    }

    /// # CHECK
    /// - The transfer out amounts must have been validated.
    pub(crate) fn unchecked_process(
        &self,
        controller: &ControllerSeeds,
        transfer_out: &TransferOut,
    ) -> Result<()> {
        let TransferOut {
            final_output_token,
            final_secondary_output_token,
            long_token,
            short_token,
            long_token_for_claimable_account_of_user,
            short_token_for_claimable_account_of_user,
            long_token_for_claimable_account_of_holding,
            short_token_for_claimable_account_of_holding,
            ..
        } = transfer_out;

        if *final_output_token != 0 {
            // Must have been validated during the execution.
            self.market_transfer_out(
                controller,
                Some(self.final_output_market.clone()),
                self.final_output_token_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                self.final_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *final_output_token,
            )?;
        }

        if *final_secondary_output_token != 0 {
            // Must have been validated during the execution.
            self.market_transfer_out(
                controller,
                Some(self.final_secondary_output_market.clone()),
                self.secondary_output_token_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                self.secondary_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *final_secondary_output_token,
            )?;
        }

        if *long_token != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.long_token_vault.to_account_info()),
                Some(self.long_token_account.to_account_info()),
                *long_token,
            )?;
        }

        if *short_token != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.short_token_vault.to_account_info()),
                Some(self.short_token_account.to_account_info()),
                *short_token,
            )?;
        }

        if *long_token_for_claimable_account_of_user != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.long_token_vault.to_account_info()),
                self.claimable_long_token_account_for_user
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *long_token_for_claimable_account_of_user,
            )?;
        }

        if *short_token_for_claimable_account_of_user != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.short_token_vault.to_account_info()),
                self.claimable_short_token_account_for_user
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *short_token_for_claimable_account_of_user,
            )?;
        }

        if *long_token_for_claimable_account_of_holding != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.long_token_vault.to_account_info()),
                self.claimable_pnl_token_account_for_holding
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *long_token_for_claimable_account_of_holding,
            )?;
        }

        if *short_token_for_claimable_account_of_holding != 0 {
            self.market_transfer_out(
                controller,
                Some(self.market.to_account_info()),
                Some(self.short_token_vault.to_account_info()),
                self.claimable_pnl_token_account_for_holding
                    .as_ref()
                    .map(|a| a.to_account_info()),
                *short_token_for_claimable_account_of_holding,
            )?;
        }

        Ok(())
    }
}
