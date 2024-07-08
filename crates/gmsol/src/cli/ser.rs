use std::{fmt, str::FromStr};

use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::types::{Factor, Market, MarketConfigKey, Pool};
use gmsol_model::{ClockKind, PoolKind};
use indexmap::IndexMap;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use strum::IntoEnumIterator;

/// Serde Factor.
#[derive(Debug, Clone)]
pub struct SerdeFactor(pub Factor);

impl fmt::Display for SerdeFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SerdeFactor {
    type Err = gmsol::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace('_', "");
        let inner = s.parse::<u128>().map_err(gmsol::Error::unknown)?;
        Ok(Self(inner))
    }
}

/// Serializable Market.
#[serde_as]
#[derive(Debug, Serialize)]
pub struct SerializeMarket {
    /// Name.
    pub name: String,
    /// Enabled.
    pub enabled: bool,
    /// Is pure.
    pub is_pure: bool,
    /// Address.
    #[serde_as(as = "DisplayFromStr")]
    pub address: Pubkey,
    /// Store.
    #[serde_as(as = "DisplayFromStr")]
    pub store: Pubkey,
    /// Meta.
    pub meta: SerializeMarketMeta,
    /// State.
    pub state: SerializeMarketState,
    /// Clocks.
    pub clocks: SerializeMarketClocks,
    /// Pools.
    pub pools: SerializeMarketPools,
    /// Parameters.
    pub params: MarketConfigMap,
}

impl SerializeMarket {
    /// Create from market.
    pub fn from_market(pubkey: &Pubkey, market: &Market) -> gmsol::Result<Self> {
        let meta = market.meta();
        let state = market.state();
        let serialized = Self {
            name: market.name()?.to_string(),
            enabled: market.is_enabled(),
            address: *pubkey,
            store: market.store,
            is_pure: market.is_pure(),
            meta: SerializeMarketMeta {
                market_token: meta.market_token_mint,
                index_token: meta.index_token_mint,
                long_token: meta.long_token_mint,
                short_token: meta.short_token_mint,
            },
            state: SerializeMarketState {
                long_token_balance: state.long_token_balance_raw(),
                short_token_balance: state.short_token_balance_raw(),
                funding_factor_per_second: state.funding_factor_per_second(),
                deposit_count: state.deposit_count(),
                withdrawal_count: state.withdrawal_count(),
                order_count: state.order_count(),
            },
            clocks: market.try_into()?,
            pools: market.try_into()?,
            params: market.try_into()?,
        };
        Ok(serialized)
    }
}

/// Serializable Market Meta.
#[serde_as]
#[derive(Debug, Serialize)]
pub struct SerializeMarketMeta {
    /// Market Token.
    #[serde_as(as = "DisplayFromStr")]
    pub market_token: Pubkey,
    /// Index Token.
    #[serde_as(as = "DisplayFromStr")]
    pub index_token: Pubkey,
    /// Long Token.
    #[serde_as(as = "DisplayFromStr")]
    pub long_token: Pubkey,
    /// Short Token.
    #[serde_as(as = "DisplayFromStr")]
    pub short_token: Pubkey,
}

/// Serializable Market Meta.
#[derive(Debug, Serialize)]
pub struct SerializeMarketState {
    /// Long token balance.
    pub long_token_balance: u64,
    /// Short token balance.
    pub short_token_balance: u64,
    /// Funding factor per second.
    pub funding_factor_per_second: i128,
    /// Deposit count.
    pub deposit_count: u64,
    /// Deposit count.
    pub withdrawal_count: u64,
    /// Deposit count.
    pub order_count: u64,
}

/// Serializable Market Clocks.
#[derive(Debug, Serialize)]
pub struct SerializeMarketClocks(pub IndexMap<ClockKind, i64>);

impl<'a> TryFrom<&'a Market> for SerializeMarketClocks {
    type Error = gmsol::Error;

    fn try_from(market: &'a Market) -> Result<Self, Self::Error> {
        let map = ClockKind::iter()
            .filter_map(|kind| market.clock(kind).map(|clock| (kind, clock)))
            .collect();
        Ok(Self(map))
    }
}

/// Serializable Market Pools.
#[derive(Debug, Serialize)]
pub struct SerializeMarketPools(pub IndexMap<PoolKind, Pool>);

impl<'a> TryFrom<&'a Market> for SerializeMarketPools {
    type Error = gmsol::Error;

    fn try_from(market: &'a Market) -> Result<Self, Self::Error> {
        let map = PoolKind::iter()
            .filter_map(|kind| market.pool(kind).map(|pool| (kind, pool)))
            .collect();
        Ok(Self(map))
    }
}

/// Market Config.
#[serde_with::serde_as]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MarketConfigMap(
    #[serde_as(as = "IndexMap<_, serde_with::DisplayFromStr>")]
    pub  IndexMap<MarketConfigKey, SerdeFactor>,
);

impl<'a> TryFrom<&'a Market> for MarketConfigMap {
    type Error = gmsol::Error;

    fn try_from(market: &'a Market) -> Result<Self, Self::Error> {
        let map = MarketConfigKey::iter()
            .map(|key| {
                let factor = market.get_config_by_key(key);
                (key, SerdeFactor(*factor))
            })
            .collect();
        Ok(Self(map))
    }
}
