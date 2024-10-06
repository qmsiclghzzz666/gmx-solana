use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use gmsol_store::cpi::accounts::MarketTransferIn;
use gmsol_store::cpi::market_transfer_in;
use gmsol_store::program::GmsolStore;

use crate::states::Controller;
use crate::utils::ControllerSeeds;

/// The accounts definition of [`fund_market`](crate::gmsol_exchange::fund_market).
#[derive(Accounts)]
pub struct FundMarket<'info> {
    /// Payer.
    pub payer: Signer<'info>,
    /// Store.
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    /// Controller.
    #[account(
        mut,
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub source: UncheckedAccount<'info>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
}

pub(crate) fn fund_market(ctx: Context<FundMarket>, amount: u64) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);
    let ctx = ctx.accounts.market_transfer_in_ctx();
    market_transfer_in(ctx.with_signer(&[&controller.as_seeds()]), amount)?;
    Ok(())
}

impl<'info> FundMarket<'info> {
    fn market_transfer_in_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MarketTransferIn<'info>> {
        CpiContext::new(
            self.store_program.to_account_info(),
            MarketTransferIn {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                from_authority: self.payer.to_account_info(),
                market: self.market.to_account_info(),
                from: self.source.to_account_info(),
                vault: self.vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
