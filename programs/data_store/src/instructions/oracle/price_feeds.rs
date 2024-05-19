use anchor_lang::prelude::*;

use crate::{
    states::{
        Config, DataStore, Oracle, PriceProvider, PriceValidator, Roles, Seed, TokenConfigMap,
    },
    utils::internal,
};

#[derive(Accounts)]
pub struct SetPricesFromPriceFeed<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [Oracle::SEED, store.key().as_ref(), &[oracle.index]],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    #[account(
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = token_config_map.bump,
    )]
    pub token_config_map: Account<'info, TokenConfigMap>,
    pub price_provider: Interface<'info, PriceProvider>,
}

/// Set the oracle prices from price feeds.
pub fn set_prices_from_price_feed<'info>(
    ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
    tokens: Vec<Pubkey>,
) -> Result<()> {
    let validator = PriceValidator::try_from(ctx.accounts.config.as_ref())?;
    ctx.accounts.oracle.set_prices_from_remaining_accounts(
        validator,
        &ctx.accounts.price_provider,
        &ctx.accounts.token_config_map,
        &tokens,
        ctx.remaining_accounts,
    )
}

impl<'info> internal::Authentication<'info> for SetPricesFromPriceFeed<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}
