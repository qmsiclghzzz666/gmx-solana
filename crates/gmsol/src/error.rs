/// Error type for `gmsol`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Empty deposit.
    #[error("empty deposit")]
    EmptyDeposit,
    /// Client Error.
    #[error("{0:?}")]
    Client(#[from] anchor_client::ClientError),
    /// Number out of range.
    #[error("numer out of range")]
    NumberOutOfRange,
    /// Unknown errors.
    #[error("unknown: {0}")]
    Unknown(String),
    /// Eyre errors.
    #[error("eyre: {0}")]
    Eyre(#[from] eyre::Error),
    /// Missing return data.
    #[error("missing return data")]
    MissingReturnData,
    /// Base64 Decode Error.
    #[error("base64: {0}")]
    Base64(#[from] base64::DecodeError),
    /// Borsh Error.
    #[error("borsh: {0}")]
    Borsh(#[from] anchor_client::anchor_lang::prelude::borsh::maybestd::io::Error),
    /// Not found.
    #[error("not found")]
    NotFound,
}

impl Error {
    /// Create unknown error.
    pub fn unknown(msg: impl ToString) -> Self {
        Self::Unknown(msg.to_string())
    }
}
