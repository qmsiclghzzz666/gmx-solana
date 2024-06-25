use anchor_lang::prelude::*;
use data_store::{cpi::accounts::RemoveWithdrawal, states::Withdrawal};

use crate::{utils::ControllerSeeds, ExchangeError};

pub(crate) struct CancelWithdrawalUtils<'a, 'info> {
    pub(super) data_store_program: AccountInfo<'info>,
    pub(super) event_authority: AccountInfo<'info>,
    pub(super) token_program: AccountInfo<'info>,
    pub(super) system_program: AccountInfo<'info>,
    pub(super) controller: AccountInfo<'info>,
    pub(super) store: AccountInfo<'info>,
    pub(super) user: AccountInfo<'info>,
    pub(super) withdrawal: &'a Account<'info, Withdrawal>,
    pub(super) market_token_account: AccountInfo<'info>,
    pub(super) market_token_vault: AccountInfo<'info>,
}

impl<'a, 'info> CancelWithdrawalUtils<'a, 'info> {
    pub(crate) fn execute(
        self,
        payer: AccountInfo<'info>,
        controller: &ControllerSeeds,
        execution_fee: u64,
    ) -> Result<()> {
        let refund = self
            .withdrawal
            .get_lamports()
            .checked_sub(execution_fee.min(super::MAX_WITHDRAWAL_EXECUTION_FEE))
            .ok_or(ExchangeError::NotEnoughExecutionFee)?;
        data_store::cpi::remove_withdrawal(
            self.remove_withdrawal_ctx(payer)
                .with_signer(&[&controller.as_seeds()]),
            refund,
        )?;
        Ok(())
    }

    fn remove_withdrawal_ctx(
        &self,
        payer: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, RemoveWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveWithdrawal {
                payer,
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                market_token: Some(self.market_token_account.to_account_info()),
                market_token_withdrawal_vault: Some(self.market_token_vault.to_account_info()),
                token_program: self.token_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }
}
