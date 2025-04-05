use gmsol_programs::anchor_lang::prelude::Error as AnchorLangError;

/// SDK Error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Anchor Error.
    #[error("anchor: {0}")]
    Anchor(Box<AnchorLangError>),
    /// Model Error.
    #[error("model: {0}")]
    Model(#[from] gmsol_model::Error),
    /// Error from [`serde_wasm_bindgen`].
    #[cfg(feature = "serde-wasm-bindgen")]
    #[error("serde-wasm: {0}")]
    SerdeWasm(#[from] serde_wasm_bindgen::Error),
    #[error("base64-decode: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

impl From<AnchorLangError> for Error {
    fn from(value: AnchorLangError) -> Self {
        Self::Anchor(Box::new(value))
    }
}

#[cfg(feature = "wasm-bindgen")]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(value: Error) -> Self {
        Self::from_str(&value.to_string())
    }
}
