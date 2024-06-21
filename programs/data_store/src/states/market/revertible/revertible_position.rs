use std::cell::RefMut;

use anchor_lang::prelude::*;

use crate::{
    constants,
    states::{position::PositionState, HasMarketMeta, Position},
    DataStoreError,
};

use super::{perp_market::RevertiblePerpMarket, Revertible};

/// Revertible Position.
pub struct RevertiblePosition<'a> {
    market: RevertiblePerpMarket<'a>,
    storage: RefMut<'a, Position>,
    state: PositionState,
    is_collateral_token_long: bool,
    is_long: bool,
}

impl<'a> RevertiblePosition<'a> {
    pub(crate) fn new<'info>(
        market: RevertiblePerpMarket<'a>,
        loader: &'a AccountLoader<'info, Position>,
    ) -> Result<Self> {
        let storage = loader.load_mut()?;
        let meta = market.market_meta();

        require_eq!(
            storage.market_token,
            meta.market_token_mint,
            DataStoreError::InvalidPositionMarket
        );

        let is_long = storage.is_long()?;
        let is_collateral_token_long = meta.to_token_side(&storage.collateral_token)?;

        Ok(Self {
            is_long,
            is_collateral_token_long,
            state: storage.state,
            market,
            storage,
        })
    }

    pub(crate) fn collateral_token(&self) -> &Pubkey {
        &self.storage.collateral_token
    }
}

impl<'a> Revertible for RevertiblePosition<'a> {
    fn commit(mut self) {
        self.market.commit();
        self.storage.state = self.state;
    }
}

impl<'a> gmx_core::Position<{ constants::MARKET_DECIMALS }> for RevertiblePosition<'a> {
    type Num = u128;

    type Signed = i128;

    type Market = RevertiblePerpMarket<'a>;

    fn market(&self) -> &Self::Market {
        &self.market
    }

    fn market_mut(&mut self) -> &mut Self::Market {
        &mut self.market
    }

    fn is_long(&self) -> bool {
        self.is_long
    }

    fn is_collateral_token_long(&self) -> bool {
        self.is_collateral_token_long
    }

    fn collateral_amount(&self) -> &Self::Num {
        &self.state.collateral_amount
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.state.collateral_amount
    }

    fn size_in_usd(&self) -> &Self::Num {
        &self.state.size_in_usd
    }

    fn size_in_tokens(&self) -> &Self::Num {
        &self.state.size_in_tokens
    }

    fn size_in_usd_mut(&mut self) -> &mut Self::Num {
        &mut self.state.size_in_usd
    }

    fn size_in_tokens_mut(&mut self) -> &mut Self::Num {
        &mut self.state.size_in_tokens
    }

    fn borrowing_factor(&self) -> &Self::Num {
        &self.state.borrowing_factor
    }

    fn borrowing_factor_mut(&mut self) -> &mut Self::Num {
        &mut self.state.borrowing_factor
    }

    fn funding_fee_amount_per_size(&self) -> &Self::Num {
        &self.state.funding_fee_amount_per_size
    }

    fn funding_fee_amount_per_size_mut(&mut self) -> &mut Self::Num {
        &mut self.state.funding_fee_amount_per_size
    }

    fn claimable_funding_fee_amount_per_size(&self, is_long_collateral: bool) -> &Self::Num {
        if is_long_collateral {
            &self.state.long_token_claimable_funding_amount_per_size
        } else {
            &self.state.short_token_claimable_funding_amount_per_size
        }
    }

    fn claimable_funding_fee_amount_per_size_mut(
        &mut self,
        is_long_collateral: bool,
    ) -> &mut Self::Num {
        if is_long_collateral {
            &mut self.state.long_token_claimable_funding_amount_per_size
        } else {
            &mut self.state.short_token_claimable_funding_amount_per_size
        }
    }

    fn increased(&mut self) -> gmx_core::Result<()> {
        let clock = Clock::get().map_err(Error::from)?;
        self.state.increased_at_slot = clock.slot;
        self.state.increased_at = clock.unix_timestamp;
        Ok(())
    }

    fn decreased(&mut self) -> gmx_core::Result<()> {
        let clock = Clock::get().map_err(Error::from)?;
        self.state.decreased_at_slot = clock.slot;
        self.state.decreased_at = clock.unix_timestamp;
        Ok(())
    }
}
