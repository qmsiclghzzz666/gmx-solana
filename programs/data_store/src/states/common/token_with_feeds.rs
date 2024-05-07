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

    /// Create an iterator of feed account metas.
    #[cfg(feature = "utils")]
    pub fn feed_account_metas(&self) -> utils::FeedAccountMetas {
        utils::FeedAccountMetas::new(self)
    }
}

#[cfg(feature = "utils")]
mod utils {
    use std::{
        iter::{Peekable, Zip},
        slice::Iter,
    };

    use super::*;

    /// Feed account metas.
    pub struct FeedAccountMetas<'a> {
        provider_with_lengths: Peekable<Zip<Iter<'a, u8>, Iter<'a, u16>>>,
        feeds: Iter<'a, Pubkey>,
        current: usize,
        failed: bool,
    }

    impl<'a> FeedAccountMetas<'a> {
        pub(super) fn new(token_with_feeds: &'a TokensWithFeed) -> Self {
            let providers = token_with_feeds.providers.iter();
            let nums = token_with_feeds.nums.iter();
            let provider_with_lengths = providers.zip(nums).peekable();
            let feeds = token_with_feeds.feeds.iter();
            Self {
                feeds,
                provider_with_lengths,
                current: 0,
                failed: false,
            }
        }
    }

    impl<'a> Iterator for FeedAccountMetas<'a> {
        type Item = Result<AccountMeta>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.failed {
                return None;
            }
            loop {
                let (provider, length) = self.provider_with_lengths.peek()?;
                if self.current == (**length as usize) {
                    self.provider_with_lengths.next();
                    self.current = 0;
                    continue;
                }
                let Ok(provider) = PriceProviderKind::try_from(**provider) else {
                    self.failed = true;
                    return Some(Err(DataStoreError::InvalidProviderKindIndex.into()));
                };
                let Some(feed) = self.feeds.next() else {
                    return Some(Err(DataStoreError::NotEnoughFeeds.into()));
                };
                let pubkey = provider.parse_feed_account(feed);
                self.current += 1;
                return Some(Ok(AccountMeta {
                    pubkey,
                    is_signer: false,
                    is_writable: false,
                }));
            }
        }
    }
}
