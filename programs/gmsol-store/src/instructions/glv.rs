use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::Mint};

use crate::{
    constants,
    states::{glv::Glv, Seed, Store},
    utils::internal,
    CoreError,
};

/// The accounts definitions for [`initialize_glv`] instruction.
#[derive(Accounts)]
#[instruction(index: u8)]
pub struct InitializeGlv<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Glv token.
    #[account(
        init,
        payer = authority,
        mint::authority = store,
        mint::decimals = constants::MARKET_TOKEN_DECIMALS,
        seeds = [
            Glv::GLV_TOKEN_SEED,
            store.key().as_ref(),
            &[index],
        ],
        bump,
        owner = token_program.key(),
    )]
    pub glv_token: InterfaceAccount<'info, Mint>,
    /// Glv account.
    #[account(
        init,
        payer = authority,
        space = 8 + Glv::INIT_SPACE,
        seeds = [
            Glv::SEED,
            glv_token.key().as_ref(),
            // Version.
            &[0],
        ],
        bump,
    )]
    pub glv: AccountLoader<'info, Glv>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

/// Initialize GLV token and account.
///
/// # CHECK
/// - Only MARKET_KEEPER is allowed to call this function.
pub(crate) fn unchecked_initialize_glv<'info>(
    ctx: Context<'_, '_, 'info, 'info, InitializeGlv<'info>>,
    index: u8,
) -> Result<()> {
    let markets = ctx.remaining_accounts;
    require_gte!(
        Glv::MAX_ALLOWED_NUMBER_OF_MARKETS,
        markets.len(),
        CoreError::ExceedMaxLengthLimit
    );
    require_gt!(markets.len(), 0, CoreError::InvalidArgument);

    let store = ctx.accounts.store.key();
    let (long_token, short_token, market_tokens) =
        Glv::process_and_validate_markets_for_init(markets, &store)?;

    ctx.accounts.glv.load_init()?.unchecked_init(
        ctx.bumps.glv,
        index,
        &store,
        &ctx.accounts.glv_token.key(),
        &long_token,
        &short_token,
        &market_tokens,
    )?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeGlv<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
