use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// Js Prices.
#[derive(Debug, Serialize, Deserialize, Tsify)]
pub struct Prices {
    /// Index token price.
    pub index_token: Price,
    /// Long token price.
    pub long_token: Price,
    /// Short token price.
    pub short_token: Price,
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

/// Js Price.
#[derive(Debug, Serialize, Deserialize, Tsify)]
pub struct Price {
    /// Min price.
    pub min: u128,
    /// Max price.
    pub max: u128,
}

impl From<Price> for gmsol_model::price::Price<u128> {
    fn from(value: Price) -> Self {
        Self {
            min: value.min,
            max: value.max,
        }
    }
}
