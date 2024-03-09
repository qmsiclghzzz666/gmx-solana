use crate::OracleError;
use anchor_lang::prelude::*;
use data_store::states::TokenConfig;

/// Check and get latest chainlink price from data feed.
pub fn check_and_get_chainlink_price<'info>(
    chainlink_program: &Program<'info, crate::Chainlink>,
    store: &Account<'info, data_store::states::DataStore>,
    token_config: &'info AccountInfo<'info>,
    feed: &AccountInfo<'info>,
    token: &Pubkey,
) -> Result<crate::Price> {
    let token_config = Account::<'info, TokenConfig>::try_from(token_config)?;
    let key = token.to_string();
    let expected_pda = token_config.pda(&store.key(), &key)?;
    require_eq!(
        expected_pda,
        token_config.key(),
        OracleError::InvalidTokenConfigAccount
    );
    require_eq!(
        token_config.price_feed,
        *feed.key,
        OracleError::InvalidPriceFeedAccount
    );
    let round =
        chainlink_solana::latest_round_data(chainlink_program.to_account_info(), feed.clone())?;
    let decimals = chainlink_solana::decimals(chainlink_program.to_account_info(), feed.clone())?;
    check_and_get_price_from_round(&round, decimals, &token_config)
}

/// Check and get price from the round data.
fn check_and_get_price_from_round(
    round: &chainlink_solana::Round,
    decimals: u8,
    token_config: &TokenConfig,
) -> Result<crate::Price> {
    let chainlink_solana::Round {
        answer, timestamp, ..
    } = round;
    require_gt!(*answer, 0, OracleError::InvalidDataFeedPrice);
    let timestamp = *timestamp as i64;
    let current = Clock::get()?.unix_timestamp;
    if current > timestamp && current - timestamp > token_config.heartbeat_duration.into() {
        return Err(OracleError::PriceFeedNotUpdated.into());
    }
    let price = crate::Decimal::try_from_price(
        *answer as u128,
        decimals,
        token_config.token_decimals,
        token_config.precision,
    )
    .map_err(|_| OracleError::InvalidDataFeedPrice)?;
    Ok(crate::Price {
        min: price,
        max: price,
    })
}
