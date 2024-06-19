use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use data_store::{
    cpi::accounts::{GetValidatedMarketMeta, MarketTransferOut, RemoveDeposit},
    program::DataStore,
    states::Deposit,
    utils::{Authenticate, Authentication},
};

use crate::{utils::ControllerSeeds, ExchangeError};

pub(crate) fn only_controller_or_deposit_creator(ctx: &Context<CancelDeposit>) -> Result<()> {
    if ctx.accounts.user.is_signer {
        // The creator is signed for the cancellation.
        Ok(())
    } else {
        // `check_role` CPI will only pass when `authority` is a signer.
        Authenticate::only_controller(ctx)
    }
}

#[derive(Accounts)]
pub struct CancelDeposit<'info> {
    /// CHECK: check by access control.
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    /// The deposit to cancel.
    ///
    /// ## Notes
    /// - Only the user who created the deposit can receive the funds,
    /// which is checked by [`remove_deposit`](data_store::instructions::remove_deposit)
    /// through CPI, who also checks whether the `store` matches.
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: check by access control.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// The token account for receiving the initial long tokens.
    #[account(mut, token::authority = user)]
    pub initial_long_token: Option<Account<'info, TokenAccount>>,
    /// The token account for receiving the initial short tokens.
    #[account(mut, token::authority = user)]
    pub initial_short_token: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub long_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub short_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_long_market: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_short_market: Option<UncheckedAccount<'info>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Cancel a deposit.
pub fn cancel_deposit(ctx: Context<CancelDeposit>, execution_fee: u64) -> Result<()> {
    // We will attach the controller seeds even it may not be provided.
    let controller = ControllerSeeds::find(ctx.accounts.store.key);

    let deposit = &ctx.accounts.deposit;
    let initial_long_token_amount = deposit.fixed.tokens.params.initial_long_token_amount;
    if initial_long_token_amount != 0 {
        data_store::cpi::market_transfer_out(
            ctx.accounts
                .market_transfer_out_ctx(true)?
                .with_signer(&[&controller.as_seeds()]),
            initial_long_token_amount,
        )?;
    }
    let initial_short_token_amount = deposit.fixed.tokens.params.initial_short_token_amount;
    if initial_short_token_amount != 0 {
        data_store::cpi::market_transfer_out(
            ctx.accounts
                .market_transfer_out_ctx(false)?
                .with_signer(&[&controller.as_seeds()]),
            initial_short_token_amount,
        )?;
    }

    let refund = ctx
        .accounts
        .deposit
        .get_lamports()
        .checked_sub(execution_fee.min(crate::MAX_DEPOSIT_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    data_store::cpi::remove_deposit(
        ctx.accounts
            .remove_deposit_ctx()
            .with_signer(&[&controller.as_seeds()]),
        refund,
    )?;

    // TODO: emit deposit removed event.
    Ok(())
}

impl<'info> Authentication<'info> for CancelDeposit<'info> {
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

impl<'info> CancelDeposit<'info> {
    fn remove_deposit_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveDeposit {
                payer: self.authority.to_account_info(),
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn get_validated_market_token_mint(&self, market: &AccountInfo<'info>) -> Result<Pubkey> {
        let mint = data_store::cpi::get_validated_market_meta(CpiContext::new(
            self.data_store_program.to_account_info(),
            GetValidatedMarketMeta {
                store: self.store.to_account_info(),
                market: market.clone(),
            },
        ))?
        .get()
        .market_token_mint;
        Ok(mint)
    }

    fn market_transfer_out_ctx(
        &self,
        is_long: bool,
    ) -> Result<CpiContext<'_, '_, '_, 'info, MarketTransferOut<'info>>> {
        let (market, to, vault) = if is_long {
            // Validate the market.
            let market = self
                .initial_long_market
                .as_ref()
                .ok_or(error!(ExchangeError::InvalidArgument))?
                .to_account_info();
            let validated_market_token = self.get_validated_market_token_mint(&market)?;
            let expected_market_token = self
                .deposit
                .dynamic
                .swap_params
                .first_market_token(true)
                .unwrap_or(&self.deposit.fixed.tokens.market_token);
            require_eq!(
                validated_market_token,
                *expected_market_token,
                ExchangeError::InvalidArgument
            );
            (
                market,
                self.initial_long_token
                    .as_ref()
                    .ok_or(error!(ExchangeError::InvalidArgument))?
                    .to_account_info(),
                self.long_token_deposit_vault
                    .as_ref()
                    .ok_or(error!(ExchangeError::InvalidArgument))?
                    .to_account_info(),
            )
        } else {
            // Validate the market.
            let market = self
                .initial_short_market
                .as_ref()
                .ok_or(error!(ExchangeError::InvalidArgument))?
                .to_account_info();
            let validated_market_token = self.get_validated_market_token_mint(&market)?;
            let expected_market_token = self
                .deposit
                .dynamic
                .swap_params
                .first_market_token(false)
                .unwrap_or(&self.deposit.fixed.tokens.market_token);
            require_eq!(
                validated_market_token,
                *expected_market_token,
                ExchangeError::InvalidArgument
            );
            (
                market,
                self.initial_short_token
                    .as_ref()
                    .ok_or(error!(ExchangeError::InvalidArgument))?
                    .to_account_info(),
                self.short_token_deposit_vault
                    .as_ref()
                    .ok_or(error!(ExchangeError::InvalidArgument))?
                    .to_account_info(),
            )
        };

        Ok(CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketTransferOut {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                market,
                to,
                vault,
                token_program: self.token_program.to_account_info(),
            },
        ))
    }
}
