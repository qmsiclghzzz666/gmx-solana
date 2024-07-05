use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use super::{
    common::{SwapParams, TokenRecord, TokensWithFeed},
    Market, NonceBytes, Seed,
};

/// Withdrawal.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Withdrawal {
    /// Fixed part.
    pub fixed: Box<Fixed>,
    /// Dynamic part.
    pub dynamic: Dynamic,
}

impl Withdrawal {
    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord], swap: &SwapParams) -> usize {
        Fixed::INIT_SPACE + Dynamic::init_space(tokens_with_feed, swap)
    }
}

/// Fixed part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Fixed {
    /// The bump seed.
    pub bump: u8,
    /// Store.
    pub store: Pubkey,
    /// The nonce bytes for this withdrawal.
    pub nonce: [u8; 32],
    /// The slot that the withdrawal was last updated at.
    pub updated_at_slot: u64,
    /// The time that the withdrawal was last updated at.
    pub updated_at: i64,
    /// The user to withdraw for.
    pub user: Pubkey,
    /// The market token account.
    pub market_token_account: Pubkey,
    /// The market on which the withdrawal will be executed.
    pub market: Pubkey,
    /// Receivers.
    pub receivers: Receivers,
    /// Tokens config.
    pub tokens: Tokens,
    reserved: [u8; 128],
}

/// Dynamic part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Dynamic {
    /// Tokens with feed.
    pub tokens_with_feed: TokensWithFeed,
    /// Swap params.
    pub swap: SwapParams,
}

impl Dynamic {
    fn init_space(tokens_with_feed: &[TokenRecord], swap: &SwapParams) -> usize {
        TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap.long_token_swap_path.len(),
                swap.short_token_swap_path.len(),
            )
    }
}

/// Fees and tokens receivers for [`Withdrawal`]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
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
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tokens {
    /// Params.
    pub params: TokenParams,
    /// The market token to burn.
    pub market_token: Pubkey,
    /// The final long token to receive.
    pub final_long_token: Pubkey,
    /// The final short token to receive.
    pub final_short_token: Pubkey,
    /// The amount of market tokens that will be withdrawn.
    pub market_token_amount: u64,
}

/// Tokens params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
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
        store: Pubkey,
        nonce: NonceBytes,
        user: Pubkey,
        market: &AccountLoader<Market>,
        market_token_account: Pubkey,
        market_token_amount: u64,
        token_params: TokenParams,
        swap_params: SwapParams,
        tokens_with_feed: Vec<TokenRecord>,
        final_long_token_receiver: &Account<TokenAccount>,
        final_short_token_receiver: &Account<TokenAccount>,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        *self = Self {
            fixed: Box::new(Fixed {
                bump,
                store,
                nonce,
                updated_at_slot: clock.slot,
                updated_at: clock.unix_timestamp,
                user,
                market_token_account,
                market: market.key(),
                receivers: Receivers {
                    ui_fee_receiver,
                    final_long_token_receiver: final_long_token_receiver.key(),
                    final_short_token_receiver: final_short_token_receiver.key(),
                },
                tokens: Tokens {
                    params: token_params,
                    market_token: market.load()?.meta().market_token_mint,
                    final_long_token: final_long_token_receiver.mint,
                    final_short_token: final_short_token_receiver.mint,
                    market_token_amount,
                },
                reserved: [0; 128],
            }),
            dynamic: Dynamic {
                tokens_with_feed: TokensWithFeed::try_from_vec(tokens_with_feed)?,
                swap: swap_params,
            },
        };
        Ok(())
    }
}
