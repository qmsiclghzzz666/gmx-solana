use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{cpi::accounts::InitializeMarket, states::DataStore};
use role_store::{Authorization, Role};

/// Decimals of market tokens.
pub const MARKET_TOKEN_DECIMALS: u8 = 18;

/// Create a new market.
pub fn create_market(ctx: Context<CreateMarket>, index_token_mint: Pubkey) -> Result<()> {
    data_store::cpi::initialize_market(
        ctx.accounts.initialize_market_ctx(),
        ctx.accounts.market_token_mint.key(),
        index_token_mint,
        ctx.accounts.long_token_mint.key(),
        ctx.accounts.short_token_mint.key(),
    )?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token_mint: Pubkey)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Role>,
    pub data_store: Account<'info, DataStore>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        mint::decimals = MARKET_TOKEN_DECIMALS,
        mint::authority = market_authority,
        seeds = [
            crate::MAREKT_TOKEN_MINT_SEED,
            data_store.key().as_ref(),
            index_token_mint.as_ref(),
            long_token_mint.key().as_ref(),
            short_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub market_token_mint: Account<'info, Mint>,
    pub long_token_mint: Account<'info, Mint>,
    pub short_token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = long_token_mint,
        token::authority = market_authority,
        seeds = [
            crate::LONG_TOKEN_SEED,
            market_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub long_token: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        token::mint = short_token_mint,
        token::authority = market_authority,
        seeds = [
            crate::SHORT_TOKEN_SEED,
            market_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub short_token: Account<'info, TokenAccount>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [], bump)]
    pub market_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, data_store::program::DataStore>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateMarket<'info> {
    fn initialize_market_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitializeMarket<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarket {
                authority: self.authority.to_account_info(),
                only_market_keeper: self.only_market_keeper.to_account_info(),
                store: self.data_store.to_account_info(),
                market: self.market.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }
}

impl<'info> Authorization<'info> for CreateMarket<'info> {
    fn role_store(&self) -> Pubkey {
        self.data_store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_market_keeper
    }
}
