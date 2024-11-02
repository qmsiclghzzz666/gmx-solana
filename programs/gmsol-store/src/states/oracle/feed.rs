use anchor_lang::prelude::*;
use gmsol_utils::InitSpace;

use crate::{states::Seed, CoreError};

use super::{price_map::SmallPrices, PriceProviderKind};

/// Custom Price Feed.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PriceFeed {
    pub(crate) bump: u8,
    pub(crate) provider: u8,
    padding_0: [u8; 2],
    pub(crate) store: Pubkey,
    pub(crate) token: Pubkey,
    pub(crate) feed_id: Pubkey,
    prices: SmallPrices,
    ts: i64,
    last_published_at_slot: u64,
    last_published_at: i64,
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

    pub(crate) fn update(&mut self, ts: i64, price: &gmsol_utils::Price) -> Result<()> {
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

        self.prices = SmallPrices::from_price(price)?;
        self.ts = ts;
        self.last_published_at_slot = slot;
        self.last_published_at = current_ts;

        Ok(())
    }
}
