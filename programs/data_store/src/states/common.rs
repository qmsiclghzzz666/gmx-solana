use anchor_lang::prelude::*;

use super::{PriceProviderKind, TokenConfig};

/// Tokens with feed.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokensWithFeed {
    /// Tokens that require prices,
    /// which must be of the same length with `feeds`.
    pub tokens: Vec<Pubkey>,
    /// Token feeds for the tokens,
    /// which must be of the same length with `tokens`.
    pub feeds: Vec<Pubkey>,
    /// Corresponding providers,
    /// which must be of the same length with `feeds`.
    pub providers: Vec<u8>,
}

/// A record of token config.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenRecord {
    token: Pubkey,
    feed: Pubkey,
    provider: u8,
}

impl TokenRecord {
    /// Create a new [`TokenRecord`]
    pub fn new(token: Pubkey, feed: Pubkey, provider: PriceProviderKind) -> Self {
        Self {
            token,
            feed,
            provider: provider as u8,
        }
    }

    /// Create a new [`TokenRecord`] from token config,
    /// using the expected provider and feed.
    pub fn from_config(token: Pubkey, config: &TokenConfig) -> Result<Self> {
        Ok(Self::new(
            token,
            config.get_expected_feed()?,
            config.expected_provider()?,
        ))
    }
}

impl TokensWithFeed {
    /// Create from token records.
    pub fn from_vec(records: Vec<TokenRecord>) -> Self {
        Self {
            tokens: records.iter().map(|r| r.token).collect(),
            feeds: records.iter().map(|r| r.feed).collect(),
            providers: records.iter().map(|r| r.provider).collect(),
        }
    }

    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord]) -> usize {
        let len = tokens_with_feed.len();
        (4 + 32 * len) + (4 + 32 * len) + (4 + len)
    }
}

/// Swap params.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SwapParams {
    /// The addresses of token mints for markets along the swap path for long token or primary token.
    ///
    /// Market addresses are not cached as they can be derived
    /// by seeding with the corresponding mint addresses.
    pub long_token_swap_path: Vec<Pubkey>,
    /// The addresses of token mints for markets along the swap path for short token or secondary token.
    ///
    /// Market addresses are not cached as they can be derived
    /// by seeding with the corresponding mint addresses.
    pub short_token_swap_path: Vec<Pubkey>,
}

impl SwapParams {
    pub(crate) fn init_space(long_path_len: usize, short_path_len: usize) -> usize {
        (4 + 32 * long_path_len) + (4 + 32 * short_path_len)
    }
}
