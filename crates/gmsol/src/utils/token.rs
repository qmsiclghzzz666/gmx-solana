use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol_store::states::{TokenMap, TokenMapAccess};
use rust_decimal::{Decimal, RoundingStrategy};

use super::unsigned_amount_to_decimal;

/// Price to min output amount.
pub fn price_to_min_output_amount(
    token_map: &TokenMap,
    token_in: &Pubkey,
    token_in_amount: u64,
    token_out: &Pubkey,
    token_in_to_token_out_price: Decimal,
) -> Option<u64> {
    let token_in = token_map.get(token_in)?;
    let token_in_amount = unsigned_amount_to_decimal(token_in_amount, token_in.token_decimals());
    let mut token_out_amount = token_in_amount.checked_div(token_in_to_token_out_price)?;
    let token_out = token_map.get(token_out)?;
    let scale = token_out.token_decimals().into();
    token_out_amount.rescale(scale);
    token_out_amount
        .round_dp_with_strategy(scale, RoundingStrategy::ToPositiveInfinity)
        .mantissa()
        .try_into()
        .ok()
}
