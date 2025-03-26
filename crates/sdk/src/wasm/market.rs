use std::sync::Arc;

use gmsol_model::{price::Prices, LiquidityMarketExt, PnlFactorKind};
use gmsol_programs::{
    anchor_lang::AccountDeserialize, gmsol_store::accounts::Market, model::MarketModel,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Wrapper of [`Market`].
#[wasm_bindgen]
pub struct JsMarket {
    market: Arc<Market>,
}

#[wasm_bindgen]
impl JsMarket {
    /// Create from account data.
    pub fn decode(mut data: &[u8]) -> crate::Result<Self> {
        let market = Market::try_deserialize(&mut data)?;

        Ok(Self {
            market: Arc::new(market),
        })
    }

    /// Convert into [`JsMarketModel`]
    pub fn to_model(&self, supply: u64) -> JsMarketModel {
        JsMarketModel {
            model: MarketModel::from_parts(self.market.clone(), supply),
        }
    }
}

/// Wrapper of [`MarketModel`].
#[wasm_bindgen]
pub struct JsMarketModel {
    model: MarketModel,
}

/// Params for calculating market token price.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketTokenPriceParams {
    /// Prices.
    pub prices: Prices<u128>,
    /// Pnl Factor.
    #[serde(default = "default_pnl_factor")]
    pub pnl_factor: PnlFactorKind,
    /// Maximize.
    pub maximize: bool,
}

fn default_pnl_factor() -> PnlFactorKind {
    PnlFactorKind::MaxAfterDeposit
}

#[wasm_bindgen]
impl JsMarketModel {
    /// Get market token price.
    pub fn market_token_price(&self, params: JsValue) -> crate::Result<u128> {
        let params: MarketTokenPriceParams = serde_wasm_bindgen::from_value(params)?;

        Ok(self
            .model
            .market_token_price(&params.prices, params.pnl_factor, params.maximize)?)
    }
}
