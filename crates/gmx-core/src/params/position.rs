/// Basic Position Parameters.
#[derive(Debug, Clone, Copy)]
pub struct PositionParams<T> {
    min_position_size_usd: T,
    min_collateral_size: T,
}

impl<T> PositionParams<T> {
    /// Create a new [`PositionParams`].
    pub fn new(min_position_size_usd: T, min_collateral_size: T) -> Self {
        Self {
            min_collateral_size,
            min_position_size_usd,
        }
    }

    /// Get min position size usd.
    pub fn min_position_size_usd(&self) -> &T {
        &self.min_position_size_usd
    }

    /// Get min collateral size.
    pub fn min_collateral_size(&self) -> &T {
        &self.min_collateral_size
    }
}
