use anchor_lang::prelude::*;

use crate::{
    states::{ops::AdlOps, Market, Oracle, PriceProvider, Store, TokenMapHeader},
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
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// Token map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Price Provider.
    pub price_provider: Interface<'info, PriceProvider>,
    /// The oracle buffer to use.
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    /// The market to update the ADL state.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
}

/// CHECK: only ORDER_KEEPER is authorized to perform this action.
pub(crate) fn unchecked_update_adl_state<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
    is_long: bool,
) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    let tokens = market
        .meta()
        .ordered_tokens()
        .into_iter()
        .collect::<Vec<_>>();

    ctx.accounts.oracle.with_prices(
        &ctx.accounts.store,
        &ctx.accounts.price_provider,
        &ctx.accounts.token_map,
        &tokens,
        ctx.remaining_accounts,
        |oracle, _remaining_accounts| market.update_adl_state(oracle, is_long),
    )?;

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
