/// Error type for `gmsol`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Client Error.
    #[error("{0:?}")]
    Client(#[from] anchor_client::ClientError),
    /// Number out of range.
    #[error("numer out of range")]
    NumberOutOfRange,
}
