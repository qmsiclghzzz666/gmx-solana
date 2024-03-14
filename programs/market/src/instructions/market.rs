use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::{InitializeMarket, InitializeMarketToken, InitializeMarketVault},
    states::DataStore,
};
use role_store::{Authorization, Role};

/// Decimals of market tokens.
pub const MARKET_TOKEN_DECIMALS: u8 = 18;

/// Create a new market.
pub fn create_market(ctx: Context<CreateMarket>, index_token_mint: Pubkey) -> Result<()> {
    data_store::cpi::initialize_market_vault(
        ctx.accounts.initialize_market_vault_ctx(true),
        Some(ctx.accounts.market_token_mint.key()),
    )?;
    data_store::cpi::initialize_market_vault(
        ctx.accounts.initialize_market_vault_ctx(false),
        Some(ctx.accounts.market_token_mint.key()),
    )?;
    data_store::cpi::initialize_market_token(
        ctx.accounts.initialize_market_token_ctx(),
        index_token_mint,
        ctx.accounts.long_token_mint.key(),
        ctx.accounts.short_token_mint.key(),
    )?;
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
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market_token_mint: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub long_token_mint: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub short_token_mint: UncheckedAccount<'info>,
    /// CHECK: check by CPI
    #[account(mut)]
    pub long_token: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub short_token: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub market_sign: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, data_store::program::DataStore>,
    pub system_program: Program<'info, System>,
    /// CHECK: check by CPI.
    pub token_program: UncheckedAccount<'info>,
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

    fn initialize_market_token_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMarketToken<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarketToken {
                authority: self.authority.to_account_info(),
                only_market_keeper: self.only_market_keeper.to_account_info(),
                store: self.data_store.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                market_sign: self.market_sign.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn initialize_market_vault_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMarketVault<'info>> {
        let (mint, vault) = if is_long_token {
            (
                self.long_token_mint.to_account_info(),
                self.long_token.to_account_info(),
            )
        } else {
            (
                self.short_token_mint.to_account_info(),
                self.short_token.to_account_info(),
            )
        };
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarketVault {
                authority: self.authority.to_account_info(),
                only_market_keeper: self.only_market_keeper.to_account_info(),
                store: self.data_store.to_account_info(),
                mint,
                vault,
                market_sign: self.market_sign.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
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
