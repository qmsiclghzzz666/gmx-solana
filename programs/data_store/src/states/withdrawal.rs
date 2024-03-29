use anchor_lang::prelude::*;

use super::{Market, NonceBytes, Seed};

const MAX_SWAP_PATH_LEN: usize = 16;

/// Withdrawal.
#[account]
#[derive(InitSpace)]
pub struct Withdrawal {
    /// The bump seed.
    pub bump: u8,
    /// The nonce bytes for this withdrawal.
    pub nonce: [u8; 32],
    /// The slot that the withdrawal was last updated at.
    pub updated_at_slot: u64,
    /// The user to withdraw for.
    pub user: Pubkey,
    /// The market on which the withdrawal will be executed.
    pub market: Pubkey,
    /// Tokens config and accounts.
    pub tokens: Tokens,
    /// Receivers.
    pub receivers: Receivers,
}

/// Fees and tokens receivers for [`Withdrawal`]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Receivers {
    /// The ui fee receiver.
    pub ui_fee_receiver: Pubkey,
    /// Token account for receiving the final long tokens.
    pub final_long_token_receiver: Pubkey,
    /// Token account for receiving the final short tokens.
    pub final_short_token_receiver: Pubkey,
}

/// Tokens config for [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Tokens {
    /// Params.
    pub params: TokenParams,
    /// The market token to burn.
    pub market_token: Pubkey,
}

/// Tokens params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct TokenParams {
    /// Final long token.
    pub final_long_token: Pubkey,
    /// Final short token.
    pub final_short_token: Pubkey,
    /// The amount of market tokens taht will be withdrawn.
    pub market_token_amount: u64,
    /// The minimum amount of final long tokens that must be withdrawn.
    pub min_long_token_amount: u64,
    /// The minimum amount of final short tokens that must be withdrawn.
    pub min_short_token_amount: u64,
    /// Swap path for long token.
    #[max_len(MAX_SWAP_PATH_LEN)]
    pub long_token_swap_path: Vec<Pubkey>,
    /// Swap path for short token.
    #[max_len(MAX_SWAP_PATH_LEN)]
    pub short_token_swap_path: Vec<Pubkey>,
    /// Whether to unwrap the native token.
    pub should_unwrap_native_token: bool,
}

impl Seed for Withdrawal {
    const SEED: &'static [u8] = b"withdrawal";
}

impl Withdrawal {
    /// The max length of swap path.
    pub const MAX_SWAP_PATH_LEN: usize = MAX_SWAP_PATH_LEN;

    pub(crate) fn init(
        &mut self,
        bump: u8,
        nonce: NonceBytes,
        user: Pubkey,
        market: &Account<Market>,
        tokens: TokenParams,
        receivers: Receivers,
    ) -> Result<()> {
        *self = Self {
            bump,
            nonce,
            updated_at_slot: Clock::get()?.slot,
            user,
            market: market.key(),
            receivers,
            tokens: Tokens::new(market, tokens),
        };
        Ok(())
    }
}

impl Tokens {
    fn new(market: &Market, params: TokenParams) -> Self {
        Self {
            params,
            market_token: market.meta.market_token_mint,
        }
    }
}
