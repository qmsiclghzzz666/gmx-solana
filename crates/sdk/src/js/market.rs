use std::sync::Arc;

use gmsol_model::{LiquidityMarketExt, PnlFactorKind};
use gmsol_programs::{
    anchor_lang::AccountDeserialize, gmsol_store::accounts::Market, model::MarketModel,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use super::price::Prices;

/// Wrapper of [`Market`].
#[wasm_bindgen(js_name = Market)]
pub struct JsMarket {
    market: Arc<Market>,
}

#[wasm_bindgen(js_class = Market)]
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

/// Params for calculating market token price.
#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MarketTokenPriceParams {
    /// Prices.
    pub prices: Prices,
    /// Pnl Factor.
    #[serde(default = "default_pnl_factor")]
    pub pnl_factor: PnlFactorKind,
    /// Maximize.
    pub maximize: bool,
}

fn default_pnl_factor() -> PnlFactorKind {
    PnlFactorKind::MaxAfterDeposit
}

/// Wrapper of [`MarketModel`].
#[wasm_bindgen(js_name = MarketModel)]
pub struct JsMarketModel {
    model: MarketModel,
}

#[wasm_bindgen(js_class = MarketModel)]
impl JsMarketModel {
    /// Get market token price.
    pub fn market_token_price(&self, params: MarketTokenPriceParams) -> crate::Result<u128> {
        Ok(self.model.market_token_price(
            &params.prices.into(),
            params.pnl_factor,
            params.maximize,
        )?)
    }
}
