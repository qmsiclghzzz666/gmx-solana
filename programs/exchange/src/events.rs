use anchor_lang::prelude::*;

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct DepositCreatedEvent {
    /// Store account.
    pub store: Pubkey,
    /// Deposit account.
    pub deposit: Pubkey,
}
