use std::cell::RefMut;

use crate::{constants, DataStoreError};
use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;

use super::AsMarket;

/// Position.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Position {
    /// Bump seed.
    pub bump: u8,
    /// Position kind (the representation of [`PositionKind`]).
    pub kind: u8,
    /// Padding.
    pub padding_0: [u8; 14],
    /// Owner.
    pub owner: Pubkey,
    /// The market token of the position market.
    pub market_token: Pubkey,
    /// Collateral token.
    pub collateral_token: Pubkey,
    /// Increased at slot.
    pub increasted_at_slot: u64,
    /// Decreased at slot.
    pub decreased_at_slot: u64,
    /// Size in tokens.
    pub size_in_tokens: u128,
    /// Collateral amount.
    pub collateral_amount: u128,
    /// Size in usd.
    pub size_in_usd: u128,
    /// Borrowing factor.
    pub borrowing_factor: u128,
    /// Funding fee amount per size.
    pub funding_fee_amount_per_size: u128,
    /// Long token claimable funding amount per size.
    pub long_token_claimable_funding_amount_per_size: u128,
    /// Short token claimable funding amount per size.
    pub short_token_claimable_funding_amount_per_size: u128,
}

impl Space for Position {
    #[allow(clippy::identity_op)]
    const INIT_SPACE: usize = (1 * 2) + (1 * 14) + (32 * 3) + (8 * 2) + (16 * 7);
}

#[cfg(test)]
const_assert_eq!(std::mem::size_of::<Position>(), Position::INIT_SPACE);

impl Position {
    /// Get position kind.
    ///
    /// Note that `Uninitialized` kind will also be returned without error.
    #[inline]
    pub fn kind_unchecked(&self) -> Result<PositionKind> {
        let kind = PositionKind::try_from_primitive(self.kind)?;
        Ok(kind)
    }

    /// Get **initialized** position kind.
    pub fn kind(&self) -> Result<PositionKind> {
        match self.kind_unchecked()? {
            PositionKind::Uninitialized => Err(DataStoreError::PositionNotInitalized.into()),
            kind => Ok(kind),
        }
    }

    /// Returns whether the position side is long.
    pub fn is_long(&self) -> Result<bool> {
        Ok(matches!(self.kind()?, PositionKind::Long))
    }
}

/// Position Kind.
#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Copy, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[num_enum(error_type(name = DataStoreError, constructor = DataStoreError::invalid_position_kind))]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum PositionKind {
    /// Uninitialized.
    Uninitialized,
    /// Long position.
    Long,
    /// Short position.
    Short,
}

/// Position Operations.
pub struct PositionOps<'a, 'info> {
    position: RefMut<'a, Position>,
    market: AsMarket<'a, 'info>,
    is_collateral_token_long: bool,
    is_long: bool,
}

impl<'a, 'info> PositionOps<'a, 'info> {
    pub(crate) fn try_new(
        market: AsMarket<'a, 'info>,
        position: &'a mut AccountLoader<'info, Position>,
    ) -> Result<Self> {
        let position = position.load_mut()?;

        let is_long = position.is_long()?;

        let meta = market.meta();
        require_eq!(
            position.market_token,
            meta.market_token_mint,
            DataStoreError::InvalidPositionMarket
        );

        let is_collateral_token_long = if meta.long_token_mint == position.collateral_token {
            true
        } else if meta.short_token_mint == position.collateral_token {
            false
        } else {
            return err!(DataStoreError::InvalidPositionCollateralToken);
        };

        Ok(Self {
            market,
            position,
            is_collateral_token_long,
            is_long,
        })
    }

    pub(crate) fn into_market(self) -> AsMarket<'a, 'info> {
        self.market
    }
}

impl<'a, 'info> gmx_core::Position<{ constants::MARKET_DECIMALS }> for PositionOps<'a, 'info> {
    type Num = u128;

    type Signed = i128;

    type Market = AsMarket<'a, 'info>;

    fn market(&self) -> &Self::Market {
        &self.market
    }

    fn market_mut(&mut self) -> &mut Self::Market {
        &mut self.market
    }

    fn is_collateral_token_long(&self) -> bool {
        self.is_collateral_token_long
    }

    fn collateral_amount(&self) -> &Self::Num {
        &self.position.collateral_amount
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.position.collateral_amount
    }

    fn size_in_usd(&self) -> &Self::Num {
        &self.position.size_in_usd
    }

    fn size_in_tokens(&self) -> &Self::Num {
        &self.position.size_in_tokens
    }

    fn size_in_usd_mut(&mut self) -> &mut Self::Num {
        &mut self.position.size_in_usd
    }

    fn size_in_tokens_mut(&mut self) -> &mut Self::Num {
        &mut self.position.size_in_tokens
    }

    fn is_long(&self) -> bool {
        self.is_long
    }
}
