use anchor_client::{solana_client::pubsub_client::PubsubClientError, solana_sdk};
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

pub use gmsol_store::StoreError;

/// Error type for `gmsol`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Empty deposit.
    #[error("empty deposit")]
    EmptyDeposit,
    /// Client Error.
    #[error("{0:#?}")]
    Client(#[from] anchor_client::ClientError),
    /// Model error.
    #[error("model: {0}")]
    Model(#[from] gmsol_model::Error),
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
    /// IO Error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// Not found.
    #[error("not found")]
    NotFound,
    /// Bytemuck error.
    #[error("bytemuck: {0}")]
    Bytemuck(bytemuck::PodCastError),
    /// Invalid Arguments.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// Format error.
    #[error("fmt: {0}")]
    Fmt(#[from] std::fmt::Error),
    /// Reqwest error.
    #[cfg(feature = "reqwest")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Parse url error.
    #[cfg(feature = "url")]
    #[error("parse url: {0}")]
    ParseUrl(#[from] url::ParseError),
    /// SSE error.
    #[cfg(all(feature = "eventsource-stream", feature = "reqwest"))]
    #[error("sse: {0}")]
    Sse(#[from] eventsource_stream::EventStreamError<reqwest::Error>),
    /// JSON error.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Decode error.
    #[cfg(feature = "decode")]
    #[error("decode: {0}")]
    Decode(#[from] gmsol_decode::DecodeError),
    /// Lagged.
    #[error("lagged: {0}")]
    Lagged(#[from] BroadcastStreamRecvError),
    /// Pubsub client closed.
    #[error("pubsub: closed")]
    PubsubClosed,
    /// Complie Solana Message Error.
    #[error("compile message: {0}")]
    CompileMessage(#[from] solana_sdk::message::CompileError),
    /// Signer Error.
    #[error("signer: {0}")]
    SignerError(#[from] solana_sdk::signer::SignerError),
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

impl From<PubsubClientError> for Error {
    fn from(err: PubsubClientError) -> Self {
        match err {
            PubsubClientError::ConnectionClosed(_) => Self::PubsubClosed,
            err => anchor_client::ClientError::from(err).into(),
        }
    }
}
