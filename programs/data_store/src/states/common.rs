use anchor_lang::prelude::*;

use crate::{utils::chunk_by::chunk_by, DataStoreError};

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
    /// Providers set,
    /// which must be of the same length with `nums`.
    pub providers: Vec<u8>,
    /// The numbers of tokens of each provider.
    pub nums: Vec<u16>,
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
    /// # Panic
    /// Panics if the number of tokens of the same provider exceeds `u16`.
    pub fn try_from_vec(mut records: Vec<TokenRecord>) -> Result<Self> {
        records.sort_by_key(|r| r.provider);
        let mut chunks = chunk_by(&records, |a, b| a.provider == b.provider);
        let mut providers = Vec::with_capacity(chunks.size_hint().0);
        let mut nums = Vec::with_capacity(chunks.size_hint().0);
        chunks.try_for_each(|chunk| {
            providers.push(chunk[0].provider);
            nums.push(
                u16::try_from(chunk.len()).map_err(|_| DataStoreError::ExceedMaxLengthLimit)?,
            );
            Result::Ok(())
        })?;
        Ok(Self {
            tokens: records.iter().map(|r| r.token).collect(),
            feeds: records.iter().map(|r| r.feed).collect(),
            providers,
            nums,
        })
    }

    // TODO: add test.
    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord]) -> usize {
        let len = tokens_with_feed.len();
        (4 + 32 * len) * 2 + (4 + len) + (4 + 2 * len)
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
