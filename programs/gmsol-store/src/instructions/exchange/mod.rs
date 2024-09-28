/// Deposit creation and cancellation.
pub mod deposit;

/// Withdrawal creation and cancellation.
pub mod withdrawal;

/// Order creation and cancellation.
pub mod order;

/// Execute Deposit.
pub mod execute_deposit;

/// Execute Withdrawal.
pub mod execute_withdrawal;

/// Execute Order.
pub mod execute_order;

/// Update ADL state.
pub mod update_adl;

/// Position cut.
pub mod position_cut;

/// Execute shift.
pub mod execute_shift;

pub use deposit::*;
pub use execute_deposit::*;
pub use execute_order::*;
pub use execute_shift::*;
pub use execute_withdrawal::*;
pub use order::*;
pub use position_cut::*;
pub use update_adl::*;
pub use withdrawal::*;

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
