/// Error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Empty deposit.
    #[error("empty deposit")]
    EmptyDeposit,
    /// Computation error.
    #[error("computation error")]
    Computation,
    /// Invalid pool value for deposit.
    #[error("invalid pool value for deposit")]
    InvalidPoolValueForDeposit,
    /// Convert error.
    #[error("convert value error")]
    Convert,
}
