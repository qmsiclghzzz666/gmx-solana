use std::{
    iter::{Peekable, Zip},
    slice::Iter,
};

use anchor_client::solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};
use data_store::states::{common::TokensWithFeed, PriceProviderKind};

use crate::pyth::find_pyth_feed_account;

/// Feeds parser.
pub struct FeedsParser<F> {
    parse: F,
}

impl<F> FeedsParser<F> {
    /// Create a new feeds parser.
    pub fn new(parse: F) -> Self {
        Self { parse }
    }

    /// Parse a [`TokensWithFeed`]
    pub fn parse<'a>(
        &'a self,
        tokens_with_feed: &'a TokensWithFeed,
    ) -> impl Iterator<Item = crate::Result<AccountMeta>> + 'a
    where
        F: Fn(&PriceProviderKind, &Pubkey) -> crate::Result<AccountMeta>,
    {
        FeedAccountMetas::new(tokens_with_feed)
            .map(|res| res.and_then(|(provider, feed)| (self.parse)(&provider, &feed)))
    }
}

/// Boxed [`FeedsParser`].
pub type BoxFeedsParser =
    FeedsParser<Box<dyn Fn(&PriceProviderKind, &Pubkey) -> crate::Result<AccountMeta>>>;

impl Default for BoxFeedsParser {
    fn default() -> Self {
        Self::new(Box::new(|provider, feed| {
            let pubkey = match provider {
                PriceProviderKind::Pyth => find_pyth_feed_account(0, feed.to_bytes()).0,
                PriceProviderKind::Chainlink | PriceProviderKind::PythLegacy => *feed,
                kind => {
                    return Err(crate::Error::invalid_argument(format!(
                        "unknown provider: {kind}"
                    )))
                }
            };
            Ok(AccountMeta {
                pubkey,
                is_signer: false,
                is_writable: false,
            })
        }))
    }
}

/// Feed account metas.
pub struct FeedAccountMetas<'a> {
    provider_with_lengths: Peekable<Zip<Iter<'a, u8>, Iter<'a, u16>>>,
    feeds: Iter<'a, Pubkey>,
    current: usize,
    failed: bool,
}

impl<'a> FeedAccountMetas<'a> {
    /// Create from [`TokensWithFeed`].
    pub fn new(token_with_feeds: &'a TokensWithFeed) -> Self {
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
    type Item = crate::Result<(PriceProviderKind, Pubkey)>;

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
                return Some(Err(crate::Error::invalid_argument(
                    "invalid provider index",
                )));
            };
            let Some(feed) = self.feeds.next() else {
                return Some(Err(crate::Error::invalid_argument("not enough feeds")));
            };
            self.current += 1;
            return Some(Ok((provider, *feed)));
        }
    }
}
