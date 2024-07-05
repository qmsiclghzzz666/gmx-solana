use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use super::{
    common::{SwapParams, TokenRecord, TokensWithFeed},
    Market, NonceBytes, Seed,
};

/// Deposit.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Deposit {
    /// Fixed part.
    pub fixed: Fixed,
    /// Dynamic part.
    pub dynamic: Dynamic,
}

impl Deposit {
    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord], swap_params: &SwapParams) -> usize {
        Fixed::INIT_SPACE + Dynamic::init_space(tokens_with_feed, swap_params)
    }
}

/// Fixed part of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Fixed {
    /// The bump seed.
    pub bump: u8,
    /// Store.
    pub store: Pubkey,
    /// The nonce bytes for this deposit.
    pub nonce: [u8; 32],
    /// The slot that the deposit was last updated at.
    pub updated_at_slot: u64,
    /// The time that the deposit was last updated at.
    pub updated_at: i64,
    /// Market.
    pub market: Pubkey,
    /// Senders.
    pub senders: Senders,
    /// The receivers of the deposit.
    pub receivers: Receivers,
    /// Tokens config.
    pub tokens: Tokens,
    reserved: [u8; 128],
}

/// Senders of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Senders {
    /// The user depositing liquidity.
    pub user: Pubkey,
    /// Initial long token account.
    pub initial_long_token_account: Option<Pubkey>,
    /// Initial short token account.
    pub initial_short_token_account: Option<Pubkey>,
}

/// Tokens config of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tokens {
    /// The market token of the market.
    pub market_token: Pubkey,
    /// Initial long token.
    pub initial_long_token: Option<Pubkey>,
    /// Initial short token.
    pub initial_short_token: Option<Pubkey>,
    /// Params.
    pub params: TokenParams,
}

/// Dynamic part of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Dynamic {
    /// Tokens with feed.
    pub tokens_with_feed: TokensWithFeed,
    /// Swap params.
    pub swap_params: SwapParams,
}

impl Dynamic {
    fn init_space(tokens_with_feed: &[TokenRecord], swap_params: &SwapParams) -> usize {
        TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap_params.long_token_swap_path.len(),
                swap_params.short_token_swap_path.len(),
            )
    }
}

impl Seed for Deposit {
    const SEED: &'static [u8] = b"deposit";
}

impl Deposit {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn init(
        &mut self,
        bump: u8,
        store: Pubkey,
        market: &AccountLoader<Market>,
        nonce: NonceBytes,
        tokens_with_feed: Vec<TokenRecord>,
        user: Pubkey,
        initial_long_token_account: Option<&Account<TokenAccount>>,
        initial_short_token_account: Option<&Account<TokenAccount>>,
        receivers: Receivers,
        token_params: TokenParams,
        swap_params: SwapParams,
    ) -> Result<()> {
        let clock = Clock::get()?;
        *self = Self {
            fixed: Fixed {
                bump,
                store,
                nonce,
                updated_at_slot: clock.slot,
                updated_at: clock.unix_timestamp,
                market: market.key(),
                senders: Senders {
                    user,
                    initial_long_token_account: initial_long_token_account
                        .as_ref()
                        .map(|a| a.key()),
                    initial_short_token_account: initial_short_token_account
                        .as_ref()
                        .map(|a| a.key()),
                },
                receivers,
                tokens: Tokens {
                    market_token: market.load()?.meta().market_token_mint,
                    initial_long_token: initial_long_token_account.as_ref().map(|a| a.mint),
                    initial_short_token: initial_short_token_account.as_ref().map(|a| a.mint),
                    params: token_params,
                },
                reserved: [0; 128],
            },
            dynamic: Dynamic {
                tokens_with_feed: TokensWithFeed::try_from_vec(tokens_with_feed)?,
                swap_params,
            },
        };
        Ok(())
    }
}

/// The receivers of the deposit.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Receivers {
    /// The address to send the liquidity tokens to.
    pub receiver: Pubkey,
    /// The ui fee receiver.
    pub ui_fee_receiver: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenParams {
    /// The amount of long tokens to deposit.
    pub initial_long_token_amount: u64,
    /// The amount of short tokens to deposit.
    pub initial_short_token_amount: u64,
    /// The minimum acceptable number of liquidity tokens.
    pub min_market_tokens: u64,
    /// Whether to unwrap the native token.
    pub should_unwrap_native_token: bool,
}
