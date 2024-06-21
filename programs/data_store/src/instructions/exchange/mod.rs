/// Execute Deposit.
pub mod execute_deposit;

/// Execute Withdrawal.
pub mod execute_withdrawal;

/// Execute Order.
pub mod execute_order;

pub use execute_deposit::*;
pub use execute_order::*;
pub use execute_withdrawal::*;

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
            gmx_core::Error::EmptyDeposit => DataStoreError::EmptyDeposit.into(),
            gmx_core::Error::Solana(err) => err,
            core_error => {
                crate::msg!("GmxCoreError occurred. Error Message: {}", core_error);
                DataStoreError::Core.into()
            }
        }
    }
}
