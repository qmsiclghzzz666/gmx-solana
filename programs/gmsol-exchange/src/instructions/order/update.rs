use anchor_lang::prelude::*;
use gmsol_store::{
    program::GmsolStore,
    states::{order::UpdateOrderParams, Market, Order},
};

use crate::ExchangeError;

/// The accounts definition for [`update_order`](crate::gmsol_exchange::update_order)
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::update_order)*
#[derive(Accounts)]
pub struct UpdateOrder<'info> {
    /// The owner of the order.
    pub user: Signer<'info>,
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
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
    let id = ctx
        .accounts
        .market
        .load_mut()?
        .state_mut()
        .next_order_id()?;
    ctx.accounts.order.update(id, params)?;
    Ok(())
}
