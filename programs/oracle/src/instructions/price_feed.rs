use anchor_lang::prelude::*;
use data_store::states::{Data, DataStore, TokenConfig};
use role_store::{Authorization, Role};

use crate::{states::Oracle, OracleError, PriceMap};

/// The Chainlink Program.
pub struct Chainlink;

impl Id for Chainlink {
    fn id() -> Pubkey {
        chainlink_solana::ID
    }
}

#[derive(Accounts)]
pub struct SetPricesFromPriceFeed<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Role>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        constraint = oracle.data_store == store.key() @ OracleError::DataStoreMismatched,
    )]
    pub oracle: Account<'info, Oracle>,
    pub chainlink_program: Program<'info, Chainlink>,
}

impl<'info> Authorization<'info> for SetPricesFromPriceFeed<'info> {
    fn role_store(&self) -> Pubkey {
        self.oracle.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_controller
    }
}

pub fn set_prices_from_price_feed<'info>(
    ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
    tokens: Vec<Pubkey>,
) -> Result<()> {
    require!(
        ctx.accounts.oracle.primary.is_empty(),
        OracleError::PricesAlreadySet
    );
    require!(
        tokens.len() <= PriceMap::MAX_TOKENS,
        OracleError::ExceedMaxTokens
    );
    // We are going to parse the remaining accounts to address accounts and feed accounts in order.
    // It won't overflow since we has checked the length before.
    let remaining = ctx.remaining_accounts;
    require!(
        (tokens.len() << 1) <= remaining.len(),
        OracleError::NotEnoughAccountInfos
    );
    // Assume the remaining accounts are arranged in the following way:
    // [address, feed; tokens.len()] [..remaining]
    for (idx, token) in tokens.iter().enumerate() {
        let token_config_idx = idx << 1;
        let feed_idx = token_config_idx + 1;
        let price = check_and_get_chainlink_price(
            &ctx.accounts.chainlink_program,
            &ctx.accounts.store,
            &remaining[token_config_idx],
            &remaining[feed_idx],
            token,
        )?;
        ctx.accounts.oracle.primary.set(token, price)?;
    }
    Ok(())
}

/// Check and get latest chainlink price from data feed.
fn check_and_get_chainlink_price<'info>(
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
