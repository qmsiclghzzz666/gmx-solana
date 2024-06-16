/// Base Market.
pub mod base;

/// Liquidity Market.
pub mod liquidity;

/// Swap Market.
pub mod swap;

/// Perpetual Market.
pub mod perp;

pub use self::{
    base::{BaseMarket, BaseMarketExt, PnlFactorKind},
    liquidity::{LiquidityMarket, LiquidityMarketExt},
    perp::{PerpMarket, PerpMarketExt},
    swap::{SwapMarket, SwapMarketExt},
};

#[inline]
fn get_msg_by_side(is_long: bool) -> &'static str {
    if is_long {
        "for long"
    } else {
        "for short"
    }
}
