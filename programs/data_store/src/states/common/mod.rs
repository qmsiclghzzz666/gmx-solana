use anchor_lang::prelude::*;

use super::{PriceProviderKind, TokenConfig};

/// Token with feeds.
pub mod token_with_feeds;

/// Dual Vec Map.
pub mod map;

pub use map::MapStore;
pub use token_with_feeds::{TokenRecord, TokensWithFeed};

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

    /// Get the first market token in the swap path.
    pub fn first_market_token(&self, is_long: bool) -> Option<&Pubkey> {
        if is_long {
            self.long_token_swap_path.first()
        } else {
            self.short_token_swap_path.first()
        }
    }

    /// Get the last market token in the swap path.
    pub fn last_market_token(&self, is_long: bool) -> Option<&Pubkey> {
        if is_long {
            self.long_token_swap_path.last()
        } else {
            self.short_token_swap_path.last()
        }
    }
}
