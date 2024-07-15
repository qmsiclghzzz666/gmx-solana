use crate::StoreError;
use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;

use super::Seed;

/// Position.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Position {
    /// Bump seed.
    pub bump: u8,
    /// Store.
    pub store: Pubkey,
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
    /// Position State.
    pub state: PositionState,
}

impl Default for Position {
    fn default() -> Self {
        use bytemuck::Zeroable;

        Self::zeroed()
    }
}

/// Position State.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[account(zero_copy)]
pub struct PositionState {
    /// Trade id.
    pub trade_id: u64,
    /// The time that the position last increased at.
    pub increased_at: i64,
    /// Updated at slot.
    pub updated_at_slot: u64,
    /// The time that the position last decreased at.
    pub decreased_at: i64,
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
    // TODO: add reserved field.
}

impl Space for Position {
    #[allow(clippy::identity_op)]
    const INIT_SPACE: usize = (1 * 2) + 32 + (1 * 14) + (32 * 3) + (8 * 4) + (16 * 7);
}

impl Seed for Position {
    const SEED: &'static [u8] = b"position";
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
            PositionKind::Uninitialized => Err(StoreError::PositionNotInitalized.into()),
            kind => Ok(kind),
        }
    }

    /// Returns whether the position side is long.
    pub fn is_long(&self) -> Result<bool> {
        Ok(matches!(self.kind()?, PositionKind::Long))
    }

    /// Initialize the position state.
    ///
    /// Returns error if `kind` is not `Unitialized`.
    pub fn try_init(
        &mut self,
        kind: PositionKind,
        bump: u8,
        store: Pubkey,
        owner: &Pubkey,
        market_token: &Pubkey,
        collateral_token: &Pubkey,
    ) -> Result<()> {
        let PositionKind::Uninitialized = self.kind_unchecked()? else {
            return err!(StoreError::PositionHasBeenInitialized);
        };
        if matches!(kind, PositionKind::Uninitialized) {
            return err!(StoreError::InvalidPositionInitailziationParams);
        }
        self.kind = kind as u8;
        self.bump = bump;
        self.store = store;
        self.padding_0 = [0; 14];
        self.owner = *owner;
        self.market_token = *market_token;
        self.collateral_token = *collateral_token;
        Ok(())
    }
}

/// Position Kind.
#[non_exhaustive]
#[repr(u8)]
#[derive(
    Clone,
    Copy,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[num_enum(error_type(name = StoreError, constructor = StoreError::invalid_position_kind))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum PositionKind {
    /// Uninitialized.
    Uninitialized,
    /// Long position.
    Long,
    /// Short position.
    Short,
}
