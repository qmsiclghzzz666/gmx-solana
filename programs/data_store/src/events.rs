use anchor_lang::prelude::*;

use crate::states::order::OrderKind;

/// Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveDepositEvent {
    /// Store.
    pub store: Pubkey,
    /// Deposit.
    pub deposit: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
}

/// Order removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveOrderEvent {
    /// Store.
    pub store: Pubkey,
    /// Kind.
    pub kind: OrderKind,
    /// Order.
    pub order: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
}

/// Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveWithdrawalEvent {
    /// Store.
    pub store: Pubkey,
    /// Withdrawal.
    pub withdrawal: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
}
