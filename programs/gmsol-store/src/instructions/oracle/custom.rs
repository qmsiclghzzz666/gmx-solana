use anchor_lang::prelude::*;
use chainlink_datastreams::interface::ChainlinkDataStreamsInterface;
use gmsol_utils::InitSpace;

use crate::{
    states::{PriceFeed, PriceFeedPrice, PriceProviderKind, Seed, Store},
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

/// CHECK: only ORDER_KEEPER is allowed to initialize price feed.
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
        &ctx.accounts.authority.key(),
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

/// The accounts definition for [`update_price_feed_with_chainlink`] instruction.
#[derive(Accounts)]
pub struct UpdatePriceFeedWithChainlink<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Verifier Account.
    /// CHECK: checked by CPI.
    pub verifier_account: UncheckedAccount<'info>,
    /// Price Feed Account.
    #[account(mut, has_one = store, has_one = authority)]
    pub price_feed: AccountLoader<'info, PriceFeed>,
    /// Chainlink Data Streams Program.
    pub chainlink: Interface<'info, ChainlinkDataStreamsInterface>,
}

/// CHECK: only ORDER_KEEPER can update custom price feed.
pub(crate) fn unchecked_update_price_feed_with_chainlink(
    ctx: Context<UpdatePriceFeedWithChainlink>,
    signed_report: Vec<u8>,
) -> Result<()> {
    let accounts = ctx.accounts;

    require_eq!(
        accounts.price_feed.load()?.provider()?,
        PriceProviderKind::ChainlinkDataStreams,
        CoreError::InvalidArgument
    );

    let price = accounts.decode_and_validate_report(&signed_report)?;

    accounts.verify_report(signed_report)?;

    accounts.price_feed.load_mut()?.update(&price)?;

    Ok(())
}

impl<'info> internal::Authentication<'info> for UpdatePriceFeedWithChainlink<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> UpdatePriceFeedWithChainlink<'info> {
    fn decode_and_validate_report(&self, signed_report: &[u8]) -> Result<PriceFeedPrice> {
        use chainlink_datastreams::report::decode;

        let report = decode(signed_report).map_err(|_| error!(CoreError::InvalidPriceReport))?;

        require_eq!(
            Pubkey::new_from_array(report.feed_id),
            self.price_feed.load()?.feed_id,
            CoreError::InvalidPriceReport
        );

        PriceFeedPrice::from_chainlink_report(&report)
    }

    fn verify_report(&self, signed_report: Vec<u8>) -> Result<()> {
        use chainlink_datastreams::interface::{verify, VerifyContext};

        let ctx = CpiContext::new(
            self.chainlink.to_account_info(),
            VerifyContext {
                verifier_account: self.verifier_account.to_account_info(),
            },
        );

        verify(ctx, signed_report)?;
        Ok(())
    }
}
