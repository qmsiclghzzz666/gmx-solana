use anchor_lang::prelude::*;
use gmsol_utils::InitSpace;

use crate::{
    states::{PriceFeed, PriceProviderKind, Seed, Store},
    utils::internal,
    CoreError,
};

/// The accounts definition for [`initialize_price_feed`] instruction.
#[derive(Accounts)]
#[instruction(provider: u8, token: Pubkey, feed_id: Pubkey)]
pub struct InitializePriceFeed<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Price feed.
    #[account(
        init,
        payer = authority,
        space = 8 + PriceFeed::INIT_SPACE,
        seeds = [PriceFeed::SEED, store.key().as_ref(), &[provider], token.as_ref(), feed_id.as_ref()],
        bump,
    )]
    pub price_feed: AccountLoader<'info, PriceFeed>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// CHECK: only MARKET_KEEPER is allowed to initialize price feed.
pub(crate) fn unchecked_initialize_price_feed(
    ctx: Context<InitializePriceFeed>,
    provider: PriceProviderKind,
    token: &Pubkey,
    feed_id: &Pubkey,
) -> Result<()> {
    require!(
        matches!(provider, PriceProviderKind::ChainlinkDataStreams),
        CoreError::NotSupportedCustomPriceProvider
    );
    let mut feed = ctx.accounts.price_feed.load_init()?;
    feed.init(
        ctx.bumps.price_feed,
        provider,
        &ctx.accounts.store.key(),
        token,
        feed_id,
    )?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializePriceFeed<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`update_price_feed`] instruction.
#[derive(Accounts)]
pub struct UpdatePriceFeed<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub price_feed: AccountLoader<'info, PriceFeed>,
}
