use anchor_lang::prelude::*;
use gmsol_store::{
    cpi::{
        accounts::GetValidatedMarketMeta,
        get_validated_market_meta,
    },
    program::GmsolStore,
    states::{Oracle, PriceProvider, Store, TokenMapHeader},
    utils::{Authentication, WithOracle, WithOracleExt, WithStore},
};

use crate::{
    utils::ControllerSeeds,
    ExchangeError,
};

/// The accounts definition for [`update_adl_state`](crate::gmsol_exchange::update_adl_state).
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::update_adl_state)*
#[derive(Accounts)]
pub struct UpdateAdlState<'info> {
    /// The address authorized to execute this instruction.
    pub authority: Signer<'info>,
    /// The controller.
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
    /// The store.
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// The authoritzed token map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Buffer for the oracle prices.
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// The market to update the ADL state.
    /// CHECK: check by the CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The price provider program.
    pub price_provider: Interface<'info, PriceProvider>,
}

/// # CHECK
/// - This instruction can only be called by the `ORDER_KEEPER` (maybe `ADL_KEEPER` in the future).
pub(crate) fn unchecked_update_adl_state<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
    is_long: bool,
) -> Result<()> {
    // Get tokens.
    let tokens = get_validated_market_meta(CpiContext::new(
        ctx.accounts.store_program.to_account_info(),
        GetValidatedMarketMeta {
            store: ctx.accounts.store.to_account_info(),
            market: ctx.accounts.market.to_account_info(),
        },
    ))?
    .get()
    .ordered_tokens();

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    ctx.accounts.with_oracle_prices(
        tokens.into_iter().collect(),
        ctx.remaining_accounts,
        &controller.as_seeds(),
        |accounts, _| accounts.update_adl_state(&controller, is_long),
    )
}

impl<'info> UpdateAdlState<'info> {
    fn update_adl_state(&self, controller: &ControllerSeeds, is_long: bool) -> Result<()> {
        use gmsol_store::cpi;

        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            cpi::accounts::UpdateAdlState {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                oracle: self.oracle.to_account_info(),
                market: self.market.to_account_info(),
            },
        );

        cpi::update_adl_state(ctx.with_signer(&[&controller.as_seeds()]), is_long)
    }
}

impl<'info> WithStore<'info> for UpdateAdlState<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for UpdateAdlState<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> WithOracle<'info> for UpdateAdlState<'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_map(&self) -> AccountInfo<'info> {
        self.token_map.to_account_info()
    }

    fn controller(&self) -> AccountInfo<'info> {
        self.controller.to_account_info()
    }
}
