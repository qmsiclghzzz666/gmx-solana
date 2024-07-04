/// Execute Deposit.
pub mod execute_deposit;

/// Execute Withdrawal.
pub mod execute_withdrawal;

/// Execute Order.
pub mod execute_order;

pub use execute_deposit::*;
pub use execute_order::*;
pub use execute_withdrawal::*;

use crate::StoreError;

pub(crate) struct ModelError(gmsol_model::Error);

impl From<gmsol_model::Error> for ModelError {
    fn from(err: gmsol_model::Error) -> Self {
        Self(err)
    }
}

impl From<ModelError> for anchor_lang::prelude::Error {
    fn from(err: ModelError) -> Self {
        match err.0 {
            gmsol_model::Error::EmptyDeposit => StoreError::EmptyDeposit.into(),
            gmsol_model::Error::Solana(err) => err,
            core_error => {
                crate::msg!("A model error occurred. Error Message: {}", core_error);
                StoreError::Model.into()
            }
        }
    }
}
