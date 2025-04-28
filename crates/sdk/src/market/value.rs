/// Min max values.
#[cfg_attr(js, derive(tsify_next::Tsify))]
#[cfg_attr(js, tsify(from_wasm_abi, into_wasm_abi))]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Value {
    /// Min value.
    pub min: u128,
    /// Max value.
    pub max: u128,
}

impl From<Value> for gmsol_model::price::Price<u128> {
    fn from(value: Value) -> Self {
        Self {
            min: value.min,
            max: value.max,
        }
    }
}

/// Min max signed values.
#[cfg_attr(js, derive(tsify_next::Tsify))]
#[cfg_attr(js, tsify(from_wasm_abi, into_wasm_abi))]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct SignedValue {
    /// Min value.
    pub min: i128,
    /// Max value.
    pub max: i128,
}
