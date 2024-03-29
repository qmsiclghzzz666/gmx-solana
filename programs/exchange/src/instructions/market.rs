use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use data_store::{
    cpi::accounts::{CheckRole, InitializeMarket, InitializeMarketToken, InitializeMarketVault},
    states::DataStore,
};
use data_store::{states::Roles, utils::Authentication};

use crate::ExchangeError;

/// Decimals of market tokens.
pub const MARKET_TOKEN_DECIMALS: u8 = 18;

/// Create a new market.
pub fn create_market(ctx: Context<CreateMarket>, index_token_mint: Pubkey) -> Result<()> {
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
    data_store::cpi::initialize_market_vault(ctx.accounts.initialize_market_vault_ctx(), None)?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token_mint: Pubkey)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Roles>,
    pub data_store: Account<'info, DataStore>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market_token_mint: UncheckedAccount<'info>,
    pub long_token_mint: Account<'info, Mint>,
    pub short_token_mint: Account<'info, Mint>,
    /// CHECK: check by CPI.
    pub market_sign: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market_token_vault: UncheckedAccount<'info>,
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
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMarketVault<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarketVault {
                authority: self.authority.to_account_info(),
                only_market_keeper: self.only_market_keeper.to_account_info(),
                store: self.data_store.to_account_info(),
                mint: self.market_token_mint.to_account_info(),
                vault: self.market_token_vault.to_account_info(),
                market_sign: self.market_sign.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

impl<'info> Authentication<'info> for CreateMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            CheckRole {
                store: self.data_store.to_account_info(),
                roles: self.only_market_keeper.to_account_info(),
            },
        )
    }

    fn on_error(&self) -> Result<()> {
        Err(ExchangeError::PermissionDenied.into())
    }
}
