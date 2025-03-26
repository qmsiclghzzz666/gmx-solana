use crate::constants;
use wasm_bindgen::prelude::*;

/// Apply `factor` to the `value`.
#[wasm_bindgen]
pub fn apply_factor(value: u128, factor: u128) -> Option<u128> {
    gmsol_model::utils::apply_factor::<_, { constants::MARKET_DECIMALS }>(&value, &factor)
}
