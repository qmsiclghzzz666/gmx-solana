use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use super::{
    common::{SwapParams, TokensWithFeed},
    Market, NonceBytes, Seed,
};

/// Withdrawal.
#[account]
pub struct Withdrawal {
    /// Fixed part.
    pub fixed: Fixed,
    /// Dynamic part.
    pub dynamic: Dynamic,
}

impl Withdrawal {
    pub(crate) fn init_space(tokens_with_feed: &[(Pubkey, Pubkey)], swap: &SwapParams) -> usize {
        Fixed::INIT_SPACE + Dynamic::init_space(tokens_with_feed, swap)
    }
}

/// Fixed part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct Fixed {
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
    /// Receivers.
    pub receivers: Receivers,
    /// Tokens config.
    pub tokens: Tokens,
}

/// Dynamic part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Dynamic {
    /// Tokens with feed.
    pub tokens_with_feed: TokensWithFeed,
    /// Swap params.
    pub swap: SwapParams,
}

impl Dynamic {
    fn init_space(tokens_with_feed: &[(Pubkey, Pubkey)], swap: &SwapParams) -> usize {
        TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap.long_token_swap_path.len(),
                swap.short_token_swap_path.len(),
            )
    }
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
    /// The final long token to receive.
    pub final_long_token: Pubkey,
    /// The final short token to receive.
    pub final_short_token: Pubkey,
    /// The amount of market tokens taht will be withdrawn.
    pub market_token_amount: u64,
}

/// Tokens params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct TokenParams {
    /// The minimum amount of final long tokens that must be withdrawn.
    pub min_long_token_amount: u64,
    /// The minimum amount of final short tokens that must be withdrawn.
    pub min_short_token_amount: u64,
    /// Whether to unwrap the native token.
    pub should_unwrap_native_token: bool,
}

impl Seed for Withdrawal {
    const SEED: &'static [u8] = b"withdrawal";
}

impl Withdrawal {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn init(
        &mut self,
        bump: u8,
        nonce: NonceBytes,
        user: Pubkey,
        market: &Account<Market>,
        market_token_amount: u64,
        token_params: TokenParams,
        swap_params: SwapParams,
        tokens_with_feed: Vec<(Pubkey, Pubkey)>,
        final_long_token_receiver: &Account<TokenAccount>,
        final_short_token_receiver: &Account<TokenAccount>,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        *self = Self {
            fixed: Fixed {
                bump,
                nonce,
                updated_at_slot: Clock::get()?.slot,
                user,
                market: market.key(),
                receivers: Receivers {
                    ui_fee_receiver,
                    final_long_token_receiver: final_long_token_receiver.key(),
                    final_short_token_receiver: final_short_token_receiver.key(),
                },
                tokens: Tokens {
                    params: token_params,
                    market_token: market.meta.market_token_mint,
                    final_long_token: final_long_token_receiver.mint,
                    final_short_token: final_short_token_receiver.mint,
                    market_token_amount,
                },
            },
            dynamic: Dynamic {
                tokens_with_feed: TokensWithFeed::from_vec(tokens_with_feed),
                swap: swap_params,
            },
        };
        Ok(())
    }
}
