use eyre::OptionExt;
use gmsol_sdk::{
    client::token_map::TokenMap,
    core::token_config::TokenMapAccess,
    programs::{anchor_lang::prelude::Pubkey, gmsol_store::accounts::Market},
    utils::{market::MarketDecimals, unsigned_amount_to_decimal, Amount, Value},
};
use rust_decimal::{Decimal, MathematicalOps};

pub(crate) fn get_token_amount_with_token_map(
    amount: &Amount,
    token: &Pubkey,
    token_map: &TokenMap,
) -> eyre::Result<u64> {
    let decimals = token_map
        .get(token)
        .ok_or_eyre("token config not found")?
        .token_decimals;
    Ok(amount.to_u64(decimals)?)
}

pub(crate) fn token_amount(
    amount: &Amount,
    token: Option<&Pubkey>,
    token_map: &TokenMap,
    market: &Market,
    is_long: bool,
) -> eyre::Result<u64> {
    let token = match token {
        Some(token) => token,
        None => {
            if is_long {
                &market.meta.long_token_mint
            } else {
                &market.meta.short_token_mint
            }
        }
    };
    get_token_amount_with_token_map(amount, token, token_map)
}

pub(crate) fn unit_price(
    price: &Value,
    token_map: &TokenMap,
    market: &Market,
) -> eyre::Result<u128> {
    let decimals = MarketDecimals::new(&market.meta.into(), token_map)?;
    let mut price = *price;
    price.0 /= Decimal::TEN.powu(decimals.index_token_decimals.into());

    Ok(price.to_u128()?)
}

/// Price to min output amount.
pub(crate) fn price_to_min_output_amount(
    token_map: &TokenMap,
    token_in: &Pubkey,
    token_in_amount: u64,
    token_out: &Pubkey,
    token_in_to_token_out_price: Value,
) -> Option<u64> {
    let token_in = token_map.get(token_in)?;
    let token_in_amount = unsigned_amount_to_decimal(token_in_amount, token_in.token_decimals());
    let token_out_amount = Amount(token_in_amount.checked_div(token_in_to_token_out_price.0)?);
    let token_out = token_map.get(token_out)?;
    token_out_amount.to_u64(token_out.token_decimals).ok()
}
