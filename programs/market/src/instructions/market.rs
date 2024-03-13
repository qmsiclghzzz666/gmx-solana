use anchor_lang::prelude::*;
use anchor_spl::token::{InitializeAccount3, Mint, Token, TokenAccount};
use data_store::{cpi::accounts::InitializeMarket, states::DataStore};
use role_store::{Authorization, Role};

/// Decimals of market tokens.
pub const MARKET_TOKEN_DECIMALS: u8 = 18;

/// Create a new market.
pub fn create_market(ctx: Context<CreateMarket>, index_token_mint: Pubkey) -> Result<()> {
    anchor_spl::token::initialize_account3(ctx.accounts.initialize_account3_ctx(true))?;
    anchor_spl::token::initialize_account3(ctx.accounts.initialize_account3_ctx(false))?;
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
    pub market_token_mint: Box<Account<'info, Mint>>,
    pub long_token_mint: Box<Account<'info, Mint>>,
    pub short_token_mint: Box<Account<'info, Mint>>,
    /// CHECK: only used to call CPI, and should check by token program.
    #[account(
        init,
        space = TokenAccount::LEN,
        payer = authority,
        // FIXME: we cannot use these constraints because of stack size limit.
        // token::mint = long_token_mint,
        // token::authority = market_authority,
        owner = token_program.key(),
        seeds = [
            crate::LONG_TOKEN_SEED,
            market_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub long_token: UncheckedAccount<'info>,
    /// CHECK: only used to call CPI, and should check by token program.
    #[account(
        init,
        space = TokenAccount::LEN,
        payer = authority,
        // FIXME: we cannot use these constraints because of stack size limit.
        // token::mint = short_token_mint,
        // token::authority = market_authority,
        owner = token_program.key(),
        seeds = [
            crate::SHORT_TOKEN_SEED,
            market_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub short_token: UncheckedAccount<'info>,
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

    fn initialize_account3_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount3<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            InitializeAccount3 {
                account: if is_long_token {
                    self.long_token.to_account_info()
                } else {
                    self.short_token.to_account_info()
                },
                mint: if is_long_token {
                    self.long_token_mint.to_account_info()
                } else {
                    self.short_token_mint.to_account_info()
                },
                authority: self.market_authority.to_account_info(),
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
