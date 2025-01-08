use anchor_lang::prelude::*;
use borsh::BorshSerialize;

use gmsol_utils::InitSpace;

use crate::states::common::action::ActionState;

use super::Event;

/// GLV Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, InitSpace)]
pub struct GlvDepositRemoved {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// GLV Deposit.
    pub glv_deposit: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// GLV token.
    pub glv_token: Pubkey,
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    #[max_len(32)]
    pub reason: String,
}

impl GlvDepositRemoved {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        glv_deposit: Pubkey,
        market_token: Pubkey,
        glv_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            glv_deposit,
            glv_token,
            market_token,
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

impl InitSpace for GlvDepositRemoved {
    const INIT_SPACE: usize = <Self as Space>::INIT_SPACE;
}

impl Event for GlvDepositRemoved {}

/// GLV Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, InitSpace)]
pub struct GlvWithdrawalRemoved {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// GLV Withdrawal
    pub glv_withdrawal: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// GLV token.
    pub glv_token: Pubkey,
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    #[max_len(32)]
    pub reason: String,
}

impl GlvWithdrawalRemoved {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        glv_withdrawal: Pubkey,
        market_token: Pubkey,
        glv_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            glv_withdrawal,
            glv_token,
            market_token,
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

impl InitSpace for GlvWithdrawalRemoved {
    const INIT_SPACE: usize = <Self as Space>::INIT_SPACE;
}

impl Event for GlvWithdrawalRemoved {}
