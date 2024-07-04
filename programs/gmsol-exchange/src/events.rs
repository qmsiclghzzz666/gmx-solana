use anchor_lang::prelude::*;

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct DepositCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Deposit account.
    pub deposit: Pubkey,
}

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WithdrawalCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Withdrawal account.
    pub withdrawal: Pubkey,
}

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OrderCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Order account.
    pub order: Pubkey,
    /// Position account.
    pub position: Option<Pubkey>,
}
