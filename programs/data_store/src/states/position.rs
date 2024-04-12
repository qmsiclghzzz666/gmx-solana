use crate::DataStoreError;
use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;

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
    pub size_in_tokens: u64,
    /// Collateral amount.
    pub collateral_amount: u64,
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

impl Position {
    /// Get position kind.
    #[inline]
    pub fn kind(&self) -> Result<PositionKind> {
        let kind = PositionKind::try_from_primitive(self.kind)?;
        Ok(kind)
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
    /// Long position.
    Long,
    /// Short position.
    Short,
}

impl Space for Position {
    #[allow(clippy::identity_op)]
    const INIT_SPACE: usize = (1 * 2) + (1 * 14) + (32 * 3) + (8 * 4) + (16 * 5);
}

#[cfg(test)]
const_assert_eq!(std::mem::size_of::<Position>(), Position::INIT_SPACE);
