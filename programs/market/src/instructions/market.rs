use anchor_lang::prelude::*;
use anchor_spl::token::{InitializeMint2, Mint, Token};
use data_store::{cpi::accounts::InitializeMarket, states::DataStore};
use role_store::{Authorization, Role};

/// Create a new market.
pub fn create_market(
    ctx: Context<CreateMarket>,
    index_token: Pubkey,
    long_token: Pubkey,
    short_token: Pubkey,
) -> Result<()> {
    anchor_spl::token::initialize_mint2(
        ctx.accounts
            .initialize_market_token_mint_ctx()
            .with_signer(&[&[&[ctx.bumps.market_token_authority]]]),
        18,
        ctx.accounts.market_token_authority.key,
        Some(ctx.accounts.market_token_authority.key),
    )?;

    data_store::cpi::initialize_market(
        ctx.accounts.initialize_market_ctx(),
        ctx.accounts.market_token.key(),
        index_token,
        long_token,
        short_token,
    )?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token: Pubkey, long_token: Pubkey, short_token: Pubkey)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Role>,
    pub data_store: Account<'info, DataStore>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(
        init,
        payer = authority,
        space = Mint::LEN,
        seeds = [
            crate::MAREKT_TOKEN_SEED,
            data_store.key().as_ref(),
            index_token.as_ref(),
            long_token.as_ref(),
            short_token.as_ref(),
        ],
        bump,
        owner = token_program.key(),
    )]
    pub market_token: UncheckedAccount<'info>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [], bump)]
    pub market_token_authority: UncheckedAccount<'info>,
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

    fn initialize_market_token_mint_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMint2<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            InitializeMint2 {
                mint: self.market_token.to_account_info(),
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
