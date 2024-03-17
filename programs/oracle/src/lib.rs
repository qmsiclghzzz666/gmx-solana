use anchor_lang::prelude::*;

use data_store::utils::Authenticate;

/// Instructions.
pub mod instructions;

use self::instructions::*;

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_prices_from_price_feed(ctx, tokens)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
        instructions::clear_all_prices(ctx)
    }
}

/// Oracle Errors.
#[error_code]
pub enum OracleError {
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
    // #[msg("Data store mismatched")]
    // DataStoreMismatched,
    #[msg("Permission denied")]
    PermissionDenied,
}
