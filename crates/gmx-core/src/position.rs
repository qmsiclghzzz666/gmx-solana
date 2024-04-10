use crate::{
    action::{decrease_position::DecreasePosition, increase_position::IncreasePosition, Prices},
    num::{MulDiv, Num, UnsignedAbs},
    Market,
};

/// A position.
pub trait Position<const DECIMALS: u8> {
    /// Unsigned number type.
    type Num: MulDiv<Signed = Self::Signed> + Num;

    /// Signed number type.
    type Signed: UnsignedAbs<Unsigned = Self::Num> + TryFrom<Self::Num> + Num;

    /// Market type.
    type Market: Market<DECIMALS, Num = Self::Num, Signed = Self::Signed>;

    /// Get a reference to the market.
    fn market(&self) -> &Self::Market;

    /// Get a mutable reference to the market.
    fn market_mut(&mut self) -> &mut Self::Market;

    /// Returns whether the collateral token is the long token of the market.
    fn is_collateral_token_long(&self) -> bool;

    /// Get a mutable reference to the collateral amount.
    fn collateral_amount_mut(&mut self) -> &mut Self::Num;

    /// Get a reference to the size (in USD) of the position.
    fn size_in_usd(&self) -> &Self::Num;

    /// Get a reference to the size (in tokens) of the position.
    fn size_in_tokens(&self) -> &Self::Num;

    /// Get a mutable reference to the size (in USD) of the position.
    fn size_in_usd_mut(&mut self) -> &mut Self::Num;

    /// Get a mutable reference to the size (in tokens) of the position.
    fn size_in_tokens_mut(&mut self) -> &mut Self::Num;

    /// Returns whether the position is a long position.
    fn is_long(&self) -> bool;
}

impl<'a, const DECIMALS: u8, P: Position<DECIMALS>> Position<DECIMALS> for &'a mut P {
    type Num = P::Num;

    type Signed = P::Signed;

    type Market = P::Market;

    fn market(&self) -> &Self::Market {
        (**self).market()
    }

    fn market_mut(&mut self) -> &mut Self::Market {
        (**self).market_mut()
    }

    fn is_collateral_token_long(&self) -> bool {
        (**self).is_collateral_token_long()
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        (**self).collateral_amount_mut()
    }

    fn size_in_usd(&self) -> &Self::Num {
        (**self).size_in_usd()
    }

    fn size_in_tokens(&self) -> &Self::Num {
        (**self).size_in_tokens()
    }

    fn size_in_usd_mut(&mut self) -> &mut Self::Num {
        (**self).size_in_usd_mut()
    }

    fn size_in_tokens_mut(&mut self) -> &mut Self::Num {
        (**self).size_in_tokens_mut()
    }

    fn is_long(&self) -> bool {
        (**self).is_long()
    }
}

/// Extension trait for [`Position`] with utils.
pub trait PositionExt<const DECIMALS: u8>: Position<DECIMALS> {
    /// Create an action to increase the position.
    fn increase(
        &mut self,
        prices: Prices<Self::Num>,
        collateral_increment_amount: Self::Num,
        size_delta_usd: Self::Num,
        acceptable_price: Option<Self::Num>,
    ) -> crate::Result<IncreasePosition<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        IncreasePosition::try_new(
            self,
            prices,
            collateral_increment_amount,
            size_delta_usd,
            acceptable_price,
        )
    }

    /// Create an action to decrease the position.
    fn decrease(
        &mut self,
        prices: Prices<Self::Num>,
        size_delta_usd: Self::Num,
        acceptable_price: Option<Self::Num>,
    ) -> crate::Result<DecreasePosition<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        DecreasePosition::try_new(self, prices, size_delta_usd, acceptable_price)
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> PositionExt<DECIMALS> for P {}
