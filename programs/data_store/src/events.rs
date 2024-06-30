use anchor_lang::prelude::*;

use crate::states::order::OrderKind;

/// Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveDepositEvent {
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Deposit.
    pub deposit: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveDepositEvent {
    pub(crate) fn new(
        store: Pubkey,
        deposit: Pubkey,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            deposit,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}

/// Order removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveOrderEvent {
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Order.
    pub order: Pubkey,
    /// Kind.
    pub kind: OrderKind,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveOrderEvent {
    pub(crate) fn new(
        store: Pubkey,
        order: Pubkey,
        kind: OrderKind,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            kind,
            order,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}

/// Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveWithdrawalEvent {
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Withdrawal.
    pub withdrawal: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveWithdrawalEvent {
    pub(crate) fn new(
        store: Pubkey,
        withdrawal: Pubkey,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            withdrawal,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}
