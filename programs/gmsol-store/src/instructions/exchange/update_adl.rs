use anchor_lang::prelude::*;

use crate::{
    states::{ops::AdlOps, Market, Oracle, Store},
    utils::internal,
};

/// The accounts definition for [`update_adl_state`](crate::gmsol_store::update_adl_state).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::update_adl_state)*
#[derive(Accounts)]
pub struct UpdateAdlState<'info> {
    /// The address authorized to execute this instruction.
    pub authority: Signer<'info>,
    /// The store that owns the market.
    pub store: AccountLoader<'info, Store>,
    /// The oracle buffer to use.
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    /// The market to update the ADL state.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
}

/// CHECK: only CONTROLLER is authorized to perform this action.
pub(crate) fn unchecked_update_adl_state(
    ctx: Context<UpdateAdlState>,
    is_long: bool,
) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    market.update_adl_state(&ctx.accounts.oracle, is_long)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for UpdateAdlState<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
