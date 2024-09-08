use anchor_lang::{err, prelude::Result};

use crate::CoreError;

/// Action State.
#[non_exhaustive]
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[num_enum(error_type(name = CoreError, constructor = CoreError::unknown_action_state))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ActionState {
    /// Pending.
    Pending,
    /// Completed.
    Completed,
    /// Cancelled.
    Cancelled,
}

impl ActionState {
    /// Transition to Completed State.
    pub fn completed(self) -> Result<Self> {
        let Self::Pending = self else {
            return err!(CoreError::PreconditionsAreNotMet);
        };
        Ok(Self::Completed)
    }

    /// Transition to Cancelled State.
    pub fn cancelled(self) -> Result<Self> {
        let Self::Pending = self else {
            return err!(CoreError::PreconditionsAreNotMet);
        };
        Ok(Self::Cancelled)
    }

    /// Check if the state is completed or cancelled.
    pub fn is_completed_or_cancelled(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }
}
