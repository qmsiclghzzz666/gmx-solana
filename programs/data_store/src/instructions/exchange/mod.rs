/// Deposit.
pub mod deposit;

pub use deposit::*;

use crate::DataStoreError;

pub(crate) struct GmxCoreError(gmx_core::Error);

impl From<gmx_core::Error> for GmxCoreError {
    fn from(err: gmx_core::Error) -> Self {
        Self(err)
    }
}

impl From<GmxCoreError> for anchor_lang::prelude::Error {
    fn from(err: GmxCoreError) -> Self {
        match err.0 {
            gmx_core::Error::EmptyDeposit => DataStoreError::InvalidArgument.into(),
            gmx_core::Error::Solana(err) => err,
            _ => DataStoreError::InvalidArgument.into(),
        }
    }
}
