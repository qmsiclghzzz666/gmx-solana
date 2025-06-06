use gmsol_programs::gmsol_store::{accounts::Glv, types::GlvMarketConfig};
use gmsol_utils::glv::GlvMarketFlag;
use indexmap::IndexMap;
use solana_sdk::pubkey::Pubkey;

use crate::utils::{GmAmount, Value};

use super::StringPubkey;

/// Serializable version of [`Glv`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeGlv {
    /// The index of the GLV.
    pub index: u16,
    /// Address of the associated store account.
    pub store: StringPubkey,
    /// Mint address of the GLV token.
    pub glv_token: StringPubkey,
    /// Mint address of the long token.
    pub long_token: StringPubkey,
    /// Mint address of the short token.
    pub short_token: StringPubkey,
    /// Unix timestamp of the last successful shift operation.
    pub shift_last_executed_at: i64,
    /// Minimum required token amount for the first deposit.
    pub min_tokens_for_first_deposit: GmAmount,
    /// Minimum interval (in seconds) required between two shift operations.
    pub shift_min_interval_secs: u32,
    /// Maximum allowed price impact factor for a shift operation.
    pub shift_max_price_impact_factor: Value,
    /// Minimum value threshold required to trigger a shift.
    pub shift_min_value: Value,
    /// Market-specific configs.
    pub markets: IndexMap<StringPubkey, SerdeGlvMarketConfig>,
}

impl SerdeGlv {
    /// Create from [`Glv`].
    pub fn from_glv(glv: &Glv) -> crate::Result<Self> {
        Ok(Self {
            index: glv.index,
            store: glv.store.into(),
            glv_token: glv.glv_token.into(),
            long_token: glv.long_token.into(),
            short_token: glv.short_token.into(),
            shift_last_executed_at: glv.shift_last_executed_at,
            min_tokens_for_first_deposit: GmAmount::from_u64(glv.min_tokens_for_first_deposit),
            shift_min_interval_secs: glv.shift_min_interval_secs,
            shift_max_price_impact_factor: Value::from_u128(glv.shift_max_price_impact_factor),
            shift_min_value: Value::from_u128(glv.shift_min_value),
            markets: glv
                .markets
                .entries()
                .map(|(k, config)| {
                    let market_token = Pubkey::new_from_array(*k);
                    Ok((
                        market_token.into(),
                        SerdeGlvMarketConfig::from_config(config)?,
                    ))
                })
                .collect::<crate::Result<_>>()?,
        })
    }
}

/// Serializable version of [`GlvMarketConfig`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeGlvMarketConfig {
    /// Maximum allowable amount of the market token.
    pub max_amount: GmAmount,
    /// Whether deposits are currently permitted for this market token.
    pub is_deposit_allowed: bool,
    /// Maximum allowable value of the market token.
    pub max_value: Value,
    /// Current balance of the market token.
    pub balance: GmAmount,
}

impl SerdeGlvMarketConfig {
    /// Create from [`GlvMarketConfig`].
    pub fn from_config(config: &GlvMarketConfig) -> crate::Result<Self> {
        Ok(Self {
            max_amount: GmAmount::from_u64(config.max_amount),
            is_deposit_allowed: config.flags.get_flag(GlvMarketFlag::IsDepositAllowed),
            max_value: Value::from_u128(config.max_value),
            balance: GmAmount::from_u64(config.balance),
        })
    }
}
