/// Error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Empty deposit.
    #[error("empty deposit")]
    EmptyDeposit,
    /// Unknown computation error.
    #[error("unknown computation error")]
    Computation,
    /// Overflow.
    #[error("overflow")]
    Overflow,
    /// Underflow.
    #[error("underflow")]
    Underflow,
    /// Invalid pool value for deposit.
    #[error("invalid pool value for deposit")]
    InvalidPoolValueForDeposit,
    /// Convert error.
    #[error("convert value error")]
    Convert,
}
