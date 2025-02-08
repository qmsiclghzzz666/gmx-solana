use anchor_lang::prelude::*;
use borsh::BorshSerialize;

use crate::states::{gt::GtState, user};

use super::Event;

/// GT updated event.
#[event]
#[cfg_attr(feature = "debug", derive(derive_more::Debug))]
#[derive(InitSpace)]
pub struct GtUpdated {
    /// Receiver.
    pub receiver: Pubkey,
    /// Whether the GT is minted or burned.
    pub is_minted: bool,
    /// Whether the GT is reward.
    pub is_reward: bool,
    /// Receiver Delta.
    pub receiver_delta: u64,
    /// Receiver balance.
    pub receiver_balance: Option<u64>,
    /// Minting cost.
    pub minting_cost: u128,
    /// Total minted.
    pub total_minted: u64,
    /// Grow steps.
    pub grow_steps: u64,
    /// Latest supply.
    pub supply: u64,
    /// Vault.
    pub vault: u64,
    #[cfg_attr(feature = "debug", debug(skip))]
    reserved: [u8; 64],
}

impl gmsol_utils::InitSpace for GtUpdated {
    const INIT_SPACE: usize = <Self as Space>::INIT_SPACE;
}

impl Event for GtUpdated {}

impl GtUpdated {
    fn new(
        receiver: Pubkey,
        minting_cost: Option<u128>,
        is_reward: bool,
        delta: u64,
        state: &GtState,
        receiver_state: Option<&user::GtState>,
    ) -> Self {
        Self {
            receiver,
            is_minted: minting_cost.is_some(),
            is_reward,
            receiver_delta: delta,
            receiver_balance: receiver_state.map(|state| state.amount),
            minting_cost: minting_cost.unwrap_or(state.minting_cost()),
            total_minted: state.total_minted(),
            grow_steps: state.grow_steps(),
            supply: state.supply(),
            vault: state.gt_vault(),
            reserved: [0; 64],
        }
    }

    /// Create a new rewarded event.
    pub fn rewarded(
        receiver: Pubkey,
        amount: u64,
        state: &GtState,
        receiver_state: Option<&user::GtState>,
    ) -> Self {
        Self::new(
            receiver,
            Some(state.minting_cost()),
            true,
            amount,
            state,
            receiver_state,
        )
    }

    /// Create a new minted event.
    pub fn minted(
        receiver: Pubkey,
        minting_cost: u128,
        amount: u64,
        state: &GtState,
        receiver_state: Option<&user::GtState>,
    ) -> Self {
        Self::new(
            receiver,
            Some(minting_cost),
            false,
            amount,
            state,
            receiver_state,
        )
    }

    /// Create a new burned event.
    pub fn burned(
        receiver: Pubkey,
        amount: u64,
        state: &GtState,
        receiver_state: Option<&user::GtState>,
    ) -> Self {
        Self::new(receiver, None, false, amount, state, receiver_state)
    }
}
