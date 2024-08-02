use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use gmsol_store::{constants::EVENT_AUTHORITY_SEED, program::GmsolStore, states::Order};

use crate::{
    states::{ActionDisabledFlag, Controller},
    utils::ControllerSeeds,
};

use super::utils::CancelOrderUtil;

/// The accounts definition for [`cancel_order`](crate::gmsol_exchange::cancel_order).
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::cancel_order)*
#[derive(Accounts)]
pub struct CancelOrder<'info> {
    /// The owner of the order.
    #[account(mut)]
    pub user: Signer<'info>,
    /// Controller.
    #[account(
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    /// The order to cancel.
    ///
    /// CHECK: Only the owner of the order can cancel the order and receive the funds,
    /// which is checked by [`remove_order`](gmsol_store::gmsol_store::remove_order)
    /// through CPI.
    #[account(mut)]
    pub order: Account<'info, Order>,
    /// CHECK: check by CPI and cancel utils.
    #[account(mut)]
    pub initial_market: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_collateral_token_account: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub initial_collateral_token_vault: Option<UncheckedAccount<'info>>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

pub(crate) fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
    ctx.accounts.controller.load()?.validate_feature_enabled(
        ctx.accounts.order.fixed.params.kind.try_into()?,
        ActionDisabledFlag::CancelOrder,
    )?;

    let controller = ControllerSeeds::find(ctx.accounts.store.key);
    ctx.accounts
        .cancel_utils()
        .execute(ctx.accounts.user.to_account_info(), &controller, 0)?;
    Ok(())
}

impl<'info> CancelOrder<'info> {
    fn cancel_utils<'a>(&'a self) -> CancelOrderUtil<'a, 'info> {
        CancelOrderUtil {
            data_store_program: self.store_program.to_account_info(),
            event_authority: self.event_authority.to_account_info(),
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
            reason: "cancel by the owner",
        }
    }
}
