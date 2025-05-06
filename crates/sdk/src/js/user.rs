use std::sync::Arc;
use wasm_bindgen::prelude::*;

use gmsol_programs::gmsol_store::accounts::UserHeader;

use crate::utils::{optional::optional_address, zero_copy::try_deserialize_zero_copy_from_base64};

/// JS binding wrapper for [`UserHeader`]
#[wasm_bindgen(js_name = User)]
pub struct JsUser {
    user: Arc<UserHeader>,
}

#[wasm_bindgen(js_class = User)]
impl JsUser {
    /// Create from base64 encoded account data.
    pub fn decode_from_base64(data: &str) -> crate::Result<Self> {
        let user = try_deserialize_zero_copy_from_base64(data)?;

        Ok(Self {
            user: Arc::new(user.0),
        })
    }

    /// Get the owner address.
    pub fn owner_address(&self) -> String {
        self.user.owner.to_string()
    }

    /// Get the store address.
    pub fn store_address(&self) -> String {
        self.user.store.to_string()
    }

    /// Get the referral code address.
    pub fn referral_code_address(&self) -> Option<String> {
        optional_address(&self.user.referral.code).map(|k| k.to_string())
    }

    /// Get the referrer address.
    pub fn referrer_address(&self) -> Option<String> {
        optional_address(&self.user.referral.referrer).map(|k| k.to_string())
    }

    /// Get the GT rank.
    pub fn gt_rank(&self) -> u8 {
        self.user.gt.rank
    }

    /// Get GT last minted at.
    pub fn gt_last_minted_at(&self) -> i64 {
        self.user.gt.last_minted_at
    }

    /// Get total minted GT amount.
    pub fn gt_total_minted(&self) -> u64 {
        self.user.gt.total_minted
    }

    /// Get GT amount.
    pub fn gt_amount(&self) -> u64 {
        self.user.gt.amount
    }

    /// Get paid fee value of GT.
    pub fn gt_paid_fee_value(&self) -> u128 {
        self.user.gt.paid_fee_value
    }

    /// Get minted fee value of GT.
    pub fn gt_minted_fee_value(&self) -> u128 {
        self.user.gt.minted_fee_value
    }
}
