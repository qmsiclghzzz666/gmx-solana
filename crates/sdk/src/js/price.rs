use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::market::Value;

/// Js Prices.
#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Prices {
    /// Index token price.
    pub index_token: Value,
    /// Long token price.
    pub long_token: Value,
    /// Short token price.
    pub short_token: Value,
}

impl From<Prices> for gmsol_model::price::Prices<u128> {
    fn from(value: Prices) -> Self {
        Self {
            index_token_price: value.index_token.into(),
            long_token_price: value.long_token.into(),
            short_token_price: value.short_token.into(),
        }
    }
}
