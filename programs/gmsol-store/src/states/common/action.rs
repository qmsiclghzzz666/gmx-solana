use anchor_lang::{err, prelude::Result, solana_program::pubkey::Pubkey};

use crate::{states::NonceBytes, CoreError};

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

/// Action Signer.
pub struct ActionSigner {
    seed: &'static [u8],
    store: Pubkey,
    owner: Pubkey,
    nonce: NonceBytes,
    bump_bytes: [u8; 1],
}

impl ActionSigner {
    /// Create a new action signer.
    pub fn new(
        seed: &'static [u8],
        store: Pubkey,
        owner: Pubkey,
        nonce: NonceBytes,
        bump: u8,
    ) -> Self {
        Self {
            seed,
            store,
            owner,
            nonce,
            bump_bytes: [bump],
        }
    }

    /// As signer seeds.
    pub fn as_seeds(&self) -> [&[u8]; 5] {
        [
            self.seed,
            self.store.as_ref(),
            self.owner.as_ref(),
            &self.nonce,
            &self.bump_bytes,
        ]
    }
}
