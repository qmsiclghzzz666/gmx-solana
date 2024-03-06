use crate::OracleError;
use anchor_lang::prelude::*;

/// Check and get latest chainlink price from data feed.
pub fn check_and_get_chainlink_price<'info>(
    chainlink_program: &Program<'info, crate::Chainlink>,
    store: &Account<'info, data_store::DataStore>,
    feed_address: &'info AccountInfo<'info>,
    feed: &AccountInfo<'info>,
    token: &Pubkey,
) -> Result<crate::Price> {
    use data_store::{keys, Address};

    let address = Account::<'info, Address>::try_from(feed_address)?;
    let key = keys::create_price_feed_key(token);
    let expected_address_key = address.create_pda(&store.key(), &key)?;
    require_eq!(
        expected_address_key,
        address.key(),
        OracleError::InvalidPriceFeedAddressAccount
    );
    require_eq!(
        address.value,
        *feed.key,
        OracleError::InvalidPriceFeedAccount
    );
    let round =
        chainlink_solana::latest_round_data(chainlink_program.to_account_info(), feed.clone())?;
    let decimals = chainlink_solana::decimals(chainlink_program.to_account_info(), feed.clone())?;
    check_and_get_price_from_round(&round, decimals)
}

/// Check and get price from the round data.
fn check_and_get_price_from_round(
    round: &chainlink_solana::Round,
    decimals: u8,
) -> Result<crate::Price> {
    require_gt!(round.answer, 0, OracleError::InvalidDataFeedPrice);
    // TODO: check the timestamp.
    // TODO: get expected precision.
    let price = crate::Decimal::try_new(
        round.answer as u128,
        decimals,
        crate::Decimal::decimal_multiplier_from_precision(decimals, 2),
    )
    .map_err(|_| OracleError::InvalidDataFeedPrice)?;
    Ok(crate::Price {
        min: price,
        max: price,
    })
}
