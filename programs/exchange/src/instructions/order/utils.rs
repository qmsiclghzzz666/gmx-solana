use anchor_lang::prelude::*;

use data_store::{
    cpi::accounts::{MarketTransferOut, RemoveOrder},
    states::Order,
};

use crate::{
    utils::{market::get_market_token_mint, ControllerSeeds},
    ExchangeError,
};

pub(crate) struct CancelOrderUtil<'a, 'info> {
    pub(super) data_store_program: AccountInfo<'info>,
    pub(super) token_program: AccountInfo<'info>,
    pub(super) system_program: AccountInfo<'info>,
    pub(super) controller: AccountInfo<'info>,
    pub(super) store: AccountInfo<'info>,
    pub(super) user: AccountInfo<'info>,
    pub(super) order: &'a Account<'info, Order>,
    pub(super) initial_market: Option<AccountInfo<'info>>,
    pub(super) initial_collateral_token_account: Option<AccountInfo<'info>>,
    pub(super) initial_collateral_token_vault: Option<AccountInfo<'info>>,
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
            data_store::cpi::market_transfer_out(
                self.market_transfer_out_ctx()?
                    .with_signer(&[&controller.as_seeds()]),
                amount_to_cancel,
            )?;
        }
        let refund = self
            .order
            .get_lamports()
            .checked_sub(execution_fee.min(crate::MAX_DEPOSIT_EXECUTION_FEE))
            .ok_or(ExchangeError::NotEnoughExecutionFee)?;
        data_store::cpi::remove_order(
            self.remove_order_ctx(payer)
                .with_signer(&[&controller.as_seeds()]),
            refund,
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
            },
        )
    }
}
