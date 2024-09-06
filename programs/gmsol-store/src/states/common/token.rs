use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

/// Token Account.
#[account(zero_copy)]
pub struct TokenAndAccount {
    /// Token.
    pub token: Pubkey,
    /// Account.
    pub account: Pubkey,
}

impl TokenAndAccount {
    /// Initialize with token account.
    pub fn init(&mut self, account: &Account<TokenAccount>) {
        self.token = account.mint;
        self.account = account.key();
    }
}
