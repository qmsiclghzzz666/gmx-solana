use anchor_lang::prelude::*;
use gmsol_store::{
    program::GmsolStore,
    utils::{Authentication, WithStore},
};

use crate::{
    states::{
        feature::{ActionDisabledFlag, DomainDisabledFlag},
        Controller,
    },
    ExchangeError,
};

/// The accounts definition for [`toggle_feature`](crate::gmsol_exchange::toggle_feature).
#[derive(Accounts)]
pub struct ToggleFeature<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    /// Controller.
    #[account(
        mut,
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
}

/// Enable or disable the given feature.
/// CHECK: only `FEATURE_KEEPER` can use this instruction.
pub(crate) fn unchecked_toggle_feature(
    ctx: Context<ToggleFeature>,
    domain: DomainDisabledFlag,
    action: ActionDisabledFlag,
    enable: bool,
) -> Result<()> {
    ctx.accounts
        .controller
        .load_mut()?
        .set_feature_disabled(domain, action, !enable);
    Ok(())
}

impl<'info> WithStore<'info> for ToggleFeature<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for ToggleFeature<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}
