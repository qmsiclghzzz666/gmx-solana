use anchor_lang::prelude::*;
use gmsol_utils::InitSpace;

use crate::{
    states::{Seed, TokenConfig},
    CoreError,
};

use super::PriceProviderKind;

/// Custom Price Feed.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PriceFeed {
    pub(crate) bump: u8,
    pub(crate) provider: u8,
    padding_0: [u8; 14],
    pub(crate) store: Pubkey,
    pub(crate) token: Pubkey,
    pub(crate) feed_id: Pubkey,
    ts: i64,
    last_published_at_slot: u64,
    last_published_at: i64,
    padding_1: [u8; 8],
    price: PriceFeedPrice,
    reserved: [u8; 256],
}

impl InitSpace for PriceFeed {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

impl Seed for PriceFeed {
    const SEED: &'static [u8] = b"price_feed";
}

impl PriceFeed {
    pub(crate) fn init(
        &mut self,
        bump: u8,
        provider: PriceProviderKind,
        store: &Pubkey,
        token: &Pubkey,
        feed_id: &Pubkey,
    ) -> Result<()> {
        self.bump = bump;
        self.provider = provider.into();
        self.store = *store;
        self.token = *token;
        self.feed_id = *feed_id;
        Ok(())
    }

    pub(crate) fn update(&mut self, ts: i64, price: &PriceFeedPrice) -> Result<()> {
        let clock = Clock::get()?;
        let slot = clock.slot;
        let current_ts = clock.unix_timestamp;

        // Validate time.
        require_gt!(
            slot,
            self.last_published_at_slot,
            CoreError::PreconditionsAreNotMet
        );
        require_gte!(
            current_ts,
            self.last_published_at,
            CoreError::PreconditionsAreNotMet
        );
        require_gte!(ts, self.ts, CoreError::InvalidArgument);

        self.ts = ts;
        self.last_published_at_slot = slot;
        self.last_published_at = current_ts;

        require_gte!(price.max_price, price.min_price, CoreError::InvalidArgument);
        require_gte!(price.max_price, price.price, CoreError::InvalidArgument);
        require_gte!(price.price, price.min_price, CoreError::InvalidArgument);
        self.price = *price;

        Ok(())
    }

    /// Get provider.
    pub fn provider(&self) -> Result<PriceProviderKind> {
        PriceProviderKind::try_from(self.provider)
            .map_err(|_| error!(CoreError::InvalidProviderKindIndex))
    }

    /// Get price feed price.
    pub fn price(&self) -> &PriceFeedPrice {
        &self.price
    }

    /// Get ts.
    pub fn ts(&self) -> i64 {
        self.ts
    }

    /// Get published slot.
    pub fn last_published_at_slot(&self) -> u64 {
        self.last_published_at_slot
    }

    pub(crate) fn check_and_get_price(
        &self,
        clock: &Clock,
        token_config: &TokenConfig,
    ) -> Result<gmsol_utils::Price> {
        let provider = self.provider()?;
        require_eq!(token_config.expected_provider()?, provider);
        let feed_id = token_config.get_feed(&provider)?;

        require_eq!(self.feed_id, feed_id, CoreError::InvalidPriceFeedAccount);

        let current = clock.unix_timestamp;
        if current > self.ts && current - self.ts > token_config.heartbeat_duration().into() {
            return Err(CoreError::PriceFeedNotUpdated.into());
        }

        self.try_to_price(token_config)
    }

    fn try_to_price(&self, token_config: &TokenConfig) -> Result<gmsol_utils::Price> {
        use gmsol_utils::price::{Decimal, Price};

        let token_decimals = token_config.token_decimals();
        let precision = token_config.precision();

        let min = Decimal::try_from_price(
            self.price.min_price,
            PriceFeedPrice::DECIMALS,
            token_decimals,
            precision,
        )
        .map_err(|_| error!(CoreError::InvalidPriceFeedPrice))?;

        let max = Decimal::try_from_price(
            self.price.max_price,
            PriceFeedPrice::DECIMALS,
            token_decimals,
            precision,
        )
        .map_err(|_| error!(CoreError::InvalidPriceFeedPrice))?;

        Ok(Price { min, max })
    }
}

/// Price structure for Price Feed.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PriceFeedPrice {
    price: u128,
    min_price: u128,
    max_price: u128,
}

impl PriceFeedPrice {
    /// Decimals.
    pub const DECIMALS: u8 = 18;

    /// Get min price.
    pub fn min_price(&self) -> &u128 {
        &self.min_price
    }

    /// Get max price.
    pub fn max_price(&self) -> &u128 {
        &self.max_price
    }

    /// Get price.
    pub fn price(&self) -> &u128 {
        &self.price
    }

    pub(crate) fn from_chainlink_report(
        report: &chainlink_datastreams::report::Report,
    ) -> Result<Self> {
        todo!()
    }
}
