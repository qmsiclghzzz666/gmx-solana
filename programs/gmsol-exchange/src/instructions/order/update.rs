use anchor_lang::prelude::*;
use gmsol_store::{
    cpi,
    program::GmsolStore,
    states::{order::UpdateOrderParams, Order},
};

use crate::{
    states::{ActionDisabledFlag, Controller},
    utils::ControllerSeeds,
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
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// The order to update.
    #[account(mut)]
    pub order: Account<'info, Order>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
}

pub(crate) fn update_order(ctx: Context<UpdateOrder>, params: UpdateOrderParams) -> Result<()> {
    ctx.accounts.controller.load()?.validate_feature_enabled(
        ctx.accounts.order.fixed.params.kind.try_into()?,
        ActionDisabledFlag::UpdateOrder,
    )?;

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    let ctx = CpiContext::new(
        ctx.accounts.store_program.to_account_info(),
        cpi::accounts::UpdateOrder {
            authority: ctx.accounts.controller.to_account_info(),
            user: ctx.accounts.user.to_account_info(),
            store: ctx.accounts.store.to_account_info(),
            market: ctx.accounts.market.to_account_info(),
            order: ctx.accounts.order.to_account_info(),
        },
    );
    cpi::update_order(ctx.with_signer(&[&controller.as_seeds()]), params)?;
    Ok(())
}
