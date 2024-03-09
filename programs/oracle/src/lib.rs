use anchor_lang::prelude::*;

use role_store::Authenticate;

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

/// Decimal type for storing prices.
pub mod decimal;

/// Price type.
pub mod price;

pub use self::{
    decimal::{Decimal, DecimalError},
    price::{Price, PriceMap},
};

use self::instructions::*;

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        instructions::initialize(ctx, key)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_prices_from_price_feed(ctx, tokens)
    }
}

/// Oracle Errors.
#[error_code]
pub enum OracleError {
    #[msg("Price of the given token already set")]
    PriceAlreadySet,
    #[msg("Prices already set")]
    PricesAlreadySet,
    #[msg("Exceed the maximum number of tokens")]
    ExceedMaxTokens,
    #[msg("Not enough account infos")]
    NotEnoughAccountInfos,
    #[msg("Invalid token config account")]
    InvalidTokenConfigAccount,
    #[msg("Invalid price feed account")]
    InvalidPriceFeedAccount,
    #[msg("Invalid price from data feed")]
    InvalidDataFeedPrice,
    #[msg("Price Feed not updated")]
    PriceFeedNotUpdated,
    #[msg("Data store mismatched")]
    DataStoreMismatched,
}
