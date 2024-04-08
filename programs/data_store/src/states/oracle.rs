use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;
use gmx_solana_utils::price::{Decimal, Price};

use crate::DataStoreError;

use super::{Seed, TokenConfig, TokenConfigMap};

/// Maximum number of tokens for a single `Price Map` to store.
const MAX_TOKENS: usize = 32;

/// Price Map.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Default)]
pub struct PriceMap {
    #[max_len(MAX_TOKENS)]
    prices: Vec<Price>,
    #[max_len(MAX_TOKENS)]
    tokens: Vec<Pubkey>,
}

impl PriceMap {
    /// Maximum number of tokens for a single `Price Map` to store.
    pub const MAX_TOKENS: usize = MAX_TOKENS;

    fn as_map(&self) -> DualVecMap<&Vec<Pubkey>, &Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&self.tokens, &self.prices)
    }

    fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<Pubkey>, &mut Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&mut self.tokens, &mut self.prices)
    }

    /// Get price of the given token key.
    pub fn get(&self, token: &Pubkey) -> Option<Price> {
        self.as_map().get(token).copied()
    }

    /// Set the price of the given token.
    /// # Error
    /// Return error if it already set.
    pub(crate) fn set(&mut self, token: &Pubkey, price: Price) -> Result<()> {
        self.as_map_mut()
            .try_insert(*token, price)
            .map_err(|_| DataStoreError::PriceAlreadySet)?;
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear(&mut self) {
        self.tokens.clear();
        self.prices.clear();
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Oracle Account.
#[account]
#[derive(InitSpace, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Oracle {
    pub bump: u8,
    pub index: u8,
    pub primary: PriceMap,
}

impl Seed for Oracle {
    const SEED: &'static [u8] = b"oracle";
}

impl Oracle {
    /// Initialize the [`Oracle`].
    pub(crate) fn init(&mut self, bump: u8, index: u8) {
        self.primary.clear();
        self.bump = bump;
        self.index = index;
    }

    /// Set prices from remaining accounts.
    pub(crate) fn set_prices_from_remaining_accounts<'info>(
        &mut self,
        price_feed_program: &Program<'info, Chainlink>,
        map: &TokenConfigMap,
        tokens: &[Pubkey],
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<()> {
        require!(self.primary.is_empty(), DataStoreError::PricesAlreadySet);
        require!(
            tokens.len() <= PriceMap::MAX_TOKENS,
            DataStoreError::ExceedMaxLengthLimit
        );
        require!(
            tokens.len() <= remaining_accounts.len(),
            ErrorCode::AccountNotEnoughKeys
        );
        // Assume the remaining accounts are arranged in the following way:
        // [token_config, feed; tokens.len()] [..remaining]
        for (idx, token) in tokens.iter().enumerate() {
            let feed = &remaining_accounts[idx];
            let map = map.as_map();
            let token_config = map
                .get(token)
                .ok_or(DataStoreError::RequiredResourceNotFound)?;
            let price = check_and_get_chainlink_price(price_feed_program, token_config, feed)?;
            self.primary.set(token, price)?;
        }
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear_all_prices(&mut self) {
        self.primary.clear();
    }

    /// Run a function inside the scope with oracle prices set.
    pub fn with_oracle_prices<'info, T>(
        &mut self,
        price_feed_program: &Program<'info, Chainlink>,
        map: &TokenConfigMap,
        tokens: &[Pubkey],
        remaining_accounts: &'info [AccountInfo<'info>],
        f: impl FnOnce(&Self, &'info [AccountInfo<'info>]) -> Result<T>,
    ) -> Result<T> {
        require_gte!(
            remaining_accounts.len(),
            tokens.len(),
            ErrorCode::AccountNotEnoughKeys
        );
        let feeds = &remaining_accounts[..tokens.len()];
        let remaining_accounts = &remaining_accounts[tokens.len()..];
        self.set_prices_from_remaining_accounts(price_feed_program, map, tokens, feeds)?;
        let output = f(self, remaining_accounts)?;
        self.clear_all_prices();
        Ok(output)
    }
}

/// The Chainlink Program.
pub struct Chainlink;

impl Id for Chainlink {
    fn id() -> Pubkey {
        chainlink_solana::ID
    }
}

/// Check and get latest chainlink price from data feed.
pub(crate) fn check_and_get_chainlink_price<'info>(
    chainlink_program: &Program<'info, Chainlink>,
    token_config: &TokenConfig,
    feed: &AccountInfo<'info>,
) -> Result<Price> {
    require!(token_config.enabled, DataStoreError::TokenConfigDisabled);
    require_eq!(
        token_config.price_feed,
        *feed.key,
        DataStoreError::InvalidPriceFeedAccount
    );
    let round =
        chainlink_solana::latest_round_data(chainlink_program.to_account_info(), feed.clone())?;
    let decimals = chainlink_solana::decimals(chainlink_program.to_account_info(), feed.clone())?;
    check_and_get_price_from_round(&round, decimals, token_config)
}

/// Check and get price from the round data.
fn check_and_get_price_from_round(
    round: &chainlink_solana::Round,
    decimals: u8,
    token_config: &TokenConfig,
) -> Result<Price> {
    let chainlink_solana::Round {
        answer, timestamp, ..
    } = round;
    require_gt!(*answer, 0, DataStoreError::InvalidPriceFeedPrice);
    let timestamp = *timestamp as i64;
    let current = Clock::get()?.unix_timestamp;
    if current > timestamp && current - timestamp > token_config.heartbeat_duration.into() {
        return Err(DataStoreError::PriceFeedNotUpdated.into());
    }
    let price = Decimal::try_from_price(
        *answer as u128,
        decimals,
        token_config.token_decimals,
        token_config.precision,
    )
    .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?;
    Ok(Price {
        min: price,
        max: price,
    })
}
