/// Base Market.
pub mod base;

/// Liquidity Market.
pub mod liquidity;

/// Swap Market.
pub mod swap;

/// Position impact utils.
pub mod position_impact;

/// Borrowing fees utils.
pub mod borrowing;

/// Perpetual Market.
pub mod perp;

pub use self::{
    base::{BaseMarket, BaseMarketExt, BaseMarketMut, BaseMarketMutExt, PnlFactorKind},
    borrowing::{BorrowingFeeMarket, BorrowingFeeMarketExt},
    liquidity::{LiquidityMarket, LiquidityMarketExt},
    perp::{PerpMarket, PerpMarketExt, PerpMarketMut, PerpMarketMutExt},
    position_impact::{
        PositionImpactMarket, PositionImpactMarketExt, PositionImpactMarketMut,
        PositionImpactMarketMutExt,
    },
    swap::{SwapMarket, SwapMarketExt, SwapMarketMut, SwapMarketMutExt},
};

#[inline]
fn get_msg_by_side(is_long: bool) -> &'static str {
    if is_long {
        "for long"
    } else {
        "for short"
    }
}
