use crate::Market;

/// A position.
pub trait Position<const DECIMALS: u8> {
    /// Unsigned number type.
    type Num;

    /// Market type.
    type Market: Market<DECIMALS, Num = Self::Num>;

    /// Get a mutable reference to the market.
    fn market_mut(&mut self) -> &mut Self::Market;

    /// Returns whether the collateral token is the long token of the market.
    fn is_collateral_token_long(&self) -> bool;

    /// Get a mutable reference to the collateral amount.
    fn collateral_amount_mut(&mut self) -> &mut Self::Num;

    /// Get a mutable reference to the size (in USD) of the position.
    fn size_in_usd_mut(&mut self) -> &mut Self::Num;

    /// Get a mutable reference to the size (in tokens) of the position.
    fn size_in_tokens_mut(&mut self) -> &mut Self::Num;

    /// Returns whether the position is a long position.
    fn is_long(&self) -> bool;
}
