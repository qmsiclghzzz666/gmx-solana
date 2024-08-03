use anchor_lang::prelude::*;

use gmsol_store::{cpi::accounts::MarketTransferOut, states::order::TransferOut};

use crate::{utils::ControllerSeeds, ExchangeError};

pub(crate) struct TransferOutUtils<'info> {
    pub(crate) store_program: AccountInfo<'info>,
    pub(crate) token_program: AccountInfo<'info>,
    pub(crate) controller: AccountInfo<'info>,
    pub(crate) market: AccountInfo<'info>,
    pub(crate) store: AccountInfo<'info>,
    pub(crate) long_token_vault: AccountInfo<'info>,
    pub(crate) long_token_account: AccountInfo<'info>,
    pub(crate) short_token_vault: AccountInfo<'info>,
    pub(crate) short_token_account: AccountInfo<'info>,
    pub(crate) final_output_token_account: Option<AccountInfo<'info>>,
    pub(crate) final_output_token_vault: Option<AccountInfo<'info>>,
    pub(crate) final_output_market: AccountInfo<'info>,
    pub(crate) secondary_output_token_account: Option<AccountInfo<'info>>,
    pub(crate) secondary_output_token_vault: Option<AccountInfo<'info>>,
    pub(crate) final_secondary_output_market: AccountInfo<'info>,
    pub(crate) claimable_long_token_account_for_user: Option<AccountInfo<'info>>,
    pub(crate) claimable_short_token_account_for_user: Option<AccountInfo<'info>>,
    pub(crate) claimable_pnl_token_account_for_holding: Option<AccountInfo<'info>>,
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
    #[inline(never)]
    pub(crate) fn unchecked_process(
        &self,
        controller: &ControllerSeeds,
        transfer_out: &mut TransferOut,
    ) -> Result<()> {
        let TransferOut {
            final_output_token,
            secondary_output_token: final_secondary_output_token,
            long_token,
            short_token,
            long_token_for_claimable_account_of_user,
            short_token_for_claimable_account_of_user,
            long_token_for_claimable_account_of_holding,
            short_token_for_claimable_account_of_holding,
            ..
        } = transfer_out;

        if *final_output_token != 0 {
            let mut merged = false;
            if self.final_output_market.key == self.market.key {
                if same_account(
                    self.final_output_token_account.as_ref(),
                    &self.long_token_account,
                ) {
                    *long_token = long_token
                        .checked_add(*final_output_token)
                        .ok_or(error!(ExchangeError::AmountOverflow))?;
                    merged = true;
                } else if same_account(
                    self.final_output_token_account.as_ref(),
                    &self.short_token_account,
                ) {
                    *short_token = short_token
                        .checked_add(*final_output_token)
                        .ok_or(error!(ExchangeError::AmountOverflow))?;
                    merged = true;
                }
            }
            if !merged {
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
        }

        if *final_secondary_output_token != 0 {
            let mut merged = false;
            if self.final_secondary_output_market.key == self.market.key {
                if same_account(
                    self.secondary_output_token_account.as_ref(),
                    &self.long_token_account,
                ) {
                    *long_token = long_token
                        .checked_add(*final_secondary_output_token)
                        .ok_or(error!(ExchangeError::AmountOverflow))?;
                    merged = true;
                } else if same_account(
                    self.secondary_output_token_account.as_ref(),
                    &self.short_token_account,
                ) {
                    *short_token = short_token
                        .checked_add(*final_secondary_output_token)
                        .ok_or(error!(ExchangeError::AmountOverflow))?;
                    merged = true;
                }
            }

            if !merged {
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
        }

        if *long_token != 0 {
            if same_account(Some(&self.long_token_account), &self.short_token_account) {
                *short_token = short_token
                    .checked_add(*long_token)
                    .ok_or(error!(ExchangeError::AmountOverflow))?;
            } else {
                self.market_transfer_out(
                    controller,
                    Some(self.market.to_account_info()),
                    Some(self.long_token_vault.to_account_info()),
                    Some(self.long_token_account.to_account_info()),
                    *long_token,
                )?;
            }
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

fn same_account<'info>(account: Option<&AccountInfo<'info>>, other: &AccountInfo<'info>) -> bool {
    match account {
        Some(account) => account.key == other.key,
        None => false,
    }
}
