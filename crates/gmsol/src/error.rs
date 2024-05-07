/// Error type for `gmsol`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Empty deposit.
    #[error("empty deposit")]
    EmptyDeposit,
    /// Client Error.
    #[error("{0:#?}")]
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
    /// Bytemuck error.
    #[error("bytemuck: {0}")]
    Bytemuck(bytemuck::PodCastError),
    /// Invalid Arguments.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

impl Error {
    /// Create unknown error.
    pub fn unknown(msg: impl ToString) -> Self {
        Self::Unknown(msg.to_string())
    }

    /// Create "invalid argument" error.
    pub fn invalid_argument(msg: impl ToString) -> Self {
        Self::InvalidArgument(msg.to_string())
    }
}

impl From<anchor_client::anchor_lang::prelude::Error> for Error {
    fn from(value: anchor_client::anchor_lang::prelude::Error) -> Self {
        Self::from(anchor_client::ClientError::from(value))
    }
}
