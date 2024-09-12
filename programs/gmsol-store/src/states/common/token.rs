use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

/// Token Account.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct TokenAndAccount {
    /// Token.
    token: Pubkey,
    /// Account.
    account: Pubkey,
}

impl TokenAndAccount {
    /// Initialize with token account.
    pub fn init(&mut self, account: &Account<TokenAccount>) {
        self.token = account.mint;
        self.account = account.key();
    }

    /// Get token.
    pub fn token(&self) -> Option<Pubkey> {
        if self.token == Pubkey::default() {
            None
        } else {
            Some(self.token)
        }
    }

    /// Get account.
    pub fn account(&self) -> Option<Pubkey> {
        if self.account == Pubkey::default() {
            None
        } else {
            Some(self.account)
        }
    }
}
