/// Base Market.
pub mod base;

/// Liquidity Market.
pub mod liquidity;

/// Swap Market.
pub mod swap;

/// Position Impact Market.
pub mod position_impact;

/// Perpetual Market.
pub mod perp;

pub use self::{
    base::{BaseMarket, BaseMarketExt, BaseMarketMut, BaseMarketMutExt, PnlFactorKind},
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
