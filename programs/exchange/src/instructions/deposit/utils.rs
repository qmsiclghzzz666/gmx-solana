use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::{MarketTransferOut, RemoveDeposit},
    states::Deposit,
};

use crate::{
    utils::{market::get_market_token_mint, ControllerSeeds},
    ExchangeError,
};

pub(super) struct TransferIn<'info> {
    from_account: AccountInfo<'info>,
    to_vault: AccountInfo<'info>,
    to_market: AccountInfo<'info>,
}

impl<'info> TransferIn<'info> {
    pub(super) fn new(
        from_account: Option<&impl AsRef<AccountInfo<'info>>>,
        to_vault: Option<&impl AsRef<AccountInfo<'info>>>,
        to_market: Option<&impl AsRef<AccountInfo<'info>>>,
    ) -> Option<Self> {
        Some(Self {
            from_account: from_account?.as_ref().clone(),
            to_vault: to_vault?.as_ref().clone(),
            to_market: to_market?.as_ref().clone(),
        })
    }

    pub(crate) fn is_from_account_not_initialized(&self) -> bool {
        crate::utils::must_be_uninitialized(&self.from_account)
    }
}

pub(super) struct CancelDepositUtils<'a, 'info> {
    pub(super) data_store_program: AccountInfo<'info>,
    pub(super) event_authority: AccountInfo<'info>,
    pub(super) token_program: AccountInfo<'info>,
    pub(super) system_program: AccountInfo<'info>,
    pub(super) store: AccountInfo<'info>,
    pub(super) controller: AccountInfo<'info>,
    pub(super) user: AccountInfo<'info>,
    pub(super) deposit: &'a Account<'info, Deposit>,
    pub(super) initial_long_token_transfer: Option<TransferIn<'info>>,
    pub(super) initial_short_token_transfer: Option<TransferIn<'info>>,
}

impl<'a, 'info> CancelDepositUtils<'a, 'info> {
    pub(super) fn execute(
        self,
        payer: AccountInfo<'info>,
        controller: &ControllerSeeds,
        execution_fee: u64,
    ) -> Result<()> {
        {
            let initial_long_token_amount =
                self.deposit.fixed.tokens.params.initial_long_token_amount;
            if initial_long_token_amount != 0 {
                data_store::cpi::market_transfer_out(
                    self.market_transfer_out_ctx(true)?
                        .with_signer(&[&controller.as_seeds()]),
                    initial_long_token_amount,
                )?;
            }
        }
        {
            let initial_short_token_amount =
                self.deposit.fixed.tokens.params.initial_short_token_amount;
            if initial_short_token_amount != 0 {
                data_store::cpi::market_transfer_out(
                    self.market_transfer_out_ctx(false)?
                        .with_signer(&[&controller.as_seeds()]),
                    initial_short_token_amount,
                )?;
            }
        }
        let refund = self
            .deposit
            .get_lamports()
            .checked_sub(execution_fee.min(crate::MAX_DEPOSIT_EXECUTION_FEE))
            .ok_or(ExchangeError::NotEnoughExecutionFee)?;
        data_store::cpi::remove_deposit(
            self.remove_deposit_ctx(payer)
                .with_signer(&[&controller.as_seeds()]),
            refund,
        )?;
        Ok(())
    }

    fn market_transfer_out_ctx(
        &self,
        is_long: bool,
    ) -> Result<CpiContext<'_, '_, '_, 'info, MarketTransferOut<'info>>> {
        let transfer_in = if is_long {
            self.initial_long_token_transfer.as_ref()
        } else {
            self.initial_short_token_transfer.as_ref()
        }
        .ok_or(error!(ExchangeError::InvalidArgument))?;
        let TransferIn {
            from_account,
            to_vault,
            to_market,
        } = transfer_in;
        let market_token = get_market_token_mint(&self.data_store_program, to_market)?;
        let expected_market_token = self
            .deposit
            .dynamic
            .swap_params
            .first_market_token(is_long)
            .unwrap_or(&self.deposit.fixed.tokens.market_token);
        require_eq!(
            market_token,
            *expected_market_token,
            ExchangeError::InvalidArgument
        );
        Ok(CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketTransferOut {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market: to_market.clone(),
                to: from_account.clone(),
                vault: to_vault.clone(),
                token_program: self.token_program.to_account_info(),
            },
        ))
    }

    fn remove_deposit_ctx(
        &self,
        payer: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, RemoveDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveDeposit {
                payer,
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }
}
