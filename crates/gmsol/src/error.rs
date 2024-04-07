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
}

impl Error {
    /// Create unknown error.
    pub fn unknown(msg: impl ToString) -> Self {
        Self::Unknown(msg.to_string())
    }
}
