use anchor_lang::prelude::*;
use gmsol_store::states::Seed;
use gmsol_utils::InitSpace;

use crate::{
    constants::CONTROLLER_SEED, states::feature::display_feature, utils::ControllerSeeds,
    ExchangeError,
};

use super::{
    feature::{ActionDisabledFlag, DisabledFeatures, DomainDisabledFlag},
    ReferralRoot,
};

/// Controller.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Controller {
    /// Bump Seed.
    pub bump: u8,
    padding_0: [u8; 15],
    /// Store.
    pub store: Pubkey,
    /// Referral root.
    root: ReferralRoot,
    /// Disabled features.
    disabled_features: DisabledFeatures,
    padding_1: [u8; 12],
    /// Reserved.
    reserved: [u8; 256],
}

impl Seed for Controller {
    const SEED: &'static [u8] = CONTROLLER_SEED;
}

impl InitSpace for Controller {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

impl Controller {
    /// As a [`ControllerSeeds`].
    pub fn as_controller_seeds(&self) -> ControllerSeeds<'_> {
        ControllerSeeds::new(&self.store, self.bump)
    }

    /// Initialize.
    pub fn init(&mut self, store: Pubkey, bump: u8) {
        self.store = store;
        self.bump = bump;
    }

    /// Get feature disabled.
    pub fn get_feature_disabled(
        &self,
        domain: DomainDisabledFlag,
        action: ActionDisabledFlag,
    ) -> Option<bool> {
        self.disabled_features.get_disabled(domain, action)
    }

    /// Is the given feature disabled.
    pub fn is_feature_disabled(
        &self,
        domain: DomainDisabledFlag,
        action: ActionDisabledFlag,
    ) -> bool {
        self.get_feature_disabled(domain, action).unwrap_or(false)
    }

    /// Validate whether the given features is enabled.
    pub fn validate_feature_enabled(
        &self,
        domain: DomainDisabledFlag,
        action: ActionDisabledFlag,
    ) -> Result<()> {
        if self.is_feature_disabled(domain, action) {
            msg!("Feature `{}` is disabled", display_feature(domain, action));
            err!(ExchangeError::FeatureDisabled)
        } else {
            Ok(())
        }
    }

    /// Set features disabled.
    pub(crate) fn set_feature_disabled(
        &mut self,
        domain: DomainDisabledFlag,
        action: ActionDisabledFlag,
        disabled: bool,
    ) {
        self.disabled_features
            .set_disabled(domain, action, disabled)
    }
}
