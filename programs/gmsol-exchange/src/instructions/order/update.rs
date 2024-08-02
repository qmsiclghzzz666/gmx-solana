use anchor_lang::prelude::*;
use gmsol_store::{
    program::GmsolStore,
    states::{order::UpdateOrderParams, Market, Order},
};

use crate::{
    states::{ActionDisabledFlag, Controller},
    ExchangeError,
};

/// The accounts definition for [`update_order`](crate::gmsol_exchange::update_order)
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::update_order)*
#[derive(Accounts)]
pub struct UpdateOrder<'info> {
    /// The owner of the order.
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
    /// CHECK: only used as an identifier.
    pub store: UncheckedAccount<'info>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The order to update.
    #[account(
        mut,
        constraint = order.fixed.store == store.key() @ ExchangeError::InvalidArgument,
        constraint = order.fixed.market == market.key() @ ExchangeError::InvalidArgument,
        constraint = order.fixed.user == user.key() @ ExchangeError::PermissionDenied,
    )]
    pub order: Account<'info, Order>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
}

pub(crate) fn update_order(ctx: Context<UpdateOrder>, params: &UpdateOrderParams) -> Result<()> {
    ctx.accounts.controller.load()?.validate_feature_enabled(
        ctx.accounts.order.fixed.params.kind.try_into()?,
        ActionDisabledFlag::UpdateOrder,
    )?;

    let id = ctx
        .accounts
        .market
        .load_mut()?
        .state_mut()
        .next_order_id()?;
    ctx.accounts.order.update(id, params)?;
    Ok(())
}
