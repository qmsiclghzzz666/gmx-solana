use anchor_lang::prelude::*;
use role_store::{Authorization, Role};

use crate::states::{Data, DataStore, Market, MarketChangeEvent};

/// Initialize the account for [`Market`].
pub fn initialize_market(
    ctx: Context<InitializeMarket>,
    index_token: Pubkey,
    long_token: Pubkey,
    short_token: Pubkey,
    market_token: Pubkey,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.bump = ctx.bumps.market;
    market.index_token = index_token;
    market.long_token = long_token;
    market.short_token = short_token;
    market.market_token = market_token;
    emit!(MarketChangeEvent {
        address: market.key(),
        init: true,
        market: (**market).clone(),
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token: Pubkey, long_token: Pubkey, short_token: Pubkey)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    only_market_keeper: Account<'info, Role>,
    store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [
            Market::SEED,
            store.key.as_ref(),
            &Market::create_key_seed(&index_token, &long_token, &short_token),
        ],
        bump,
    )]
    market: Account<'info, Market>,
    system_program: Program<'info, System>,
}

impl<'info> Authorization<'info> for InitializeMarket<'info> {
    fn role_store(&self) -> Pubkey {
        self.store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_market_keeper
    }
}

/// Update market.
pub fn update_market(ctx: Context<UpdateMarket>, market_token: Pubkey) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.market_token = market_token;
    emit!(MarketChangeEvent {
        address: market.key(),
        init: false,
        market: (**market).clone(),
    });
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateMarket<'info> {
    authority: Signer<'info>,
    only_market_keeper: Account<'info, Role>,
    store: Account<'info, DataStore>,
    #[account(
        mut,
        seeds = [Market::SEED, store.key.as_ref(), &market.expected_key_seed()],
        bump = market.bump,
    )]
    market: Account<'info, Market>,
}

impl<'info> Authorization<'info> for UpdateMarket<'info> {
    fn role_store(&self) -> Pubkey {
        self.store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_market_keeper
    }
}
