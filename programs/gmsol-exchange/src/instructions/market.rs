use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use gmsol_store::cpi::accounts::{InitializeMarket, InitializeMarketToken, InitializeMarketVault};
use gmsol_store::program::GmsolStore;
use gmsol_store::utils::{Authentication, WithStore};

use crate::ExchangeError;

/// Create a new market.
pub fn create_market(
    ctx: Context<CreateMarket>,
    name: String,
    index_token_mint: Pubkey,
    enable: bool,
) -> Result<()> {
    gmsol_store::cpi::initialize_market_token(
        ctx.accounts.initialize_market_token_ctx(),
        index_token_mint,
        ctx.accounts.long_token_mint.key(),
        ctx.accounts.short_token_mint.key(),
    )?;
    gmsol_store::cpi::initialize_market(
        ctx.accounts.initialize_market_ctx(),
        ctx.accounts.market_token_mint.key(),
        index_token_mint,
        ctx.accounts.long_token_mint.key(),
        ctx.accounts.short_token_mint.key(),
        name,
        enable,
    )?;
    gmsol_store::cpi::initialize_market_vault(
        ctx.accounts.initialize_market_vault_ctx(TokenKind::Long),
        None,
    )?;
    gmsol_store::cpi::initialize_market_vault(
        ctx.accounts.initialize_market_vault_ctx(TokenKind::Short),
        None,
    )?;
    gmsol_store::cpi::initialize_market_vault(
        ctx.accounts.initialize_market_vault_ctx(TokenKind::Market),
        None,
    )?;
    Ok(())
}

enum TokenKind {
    Market,
    Long,
    Short,
}

#[derive(Accounts)]
#[instruction(index_token_mint: Pubkey)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: check by CPI.
    pub data_store: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub token_map: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market_token_mint: UncheckedAccount<'info>,
    pub long_token_mint: Account<'info, Mint>,
    pub short_token_mint: Account<'info, Mint>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub market_token_vault: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub long_token_vault: UncheckedAccount<'info>,
    /// CHECK: check and init by CPI.
    #[account(mut)]
    pub short_token_vault: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateMarket<'info> {
    fn initialize_market_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitializeMarket<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarket {
                authority: self.authority.to_account_info(),
                store: self.data_store.to_account_info(),
                token_map: self.token_map.to_account_info(),
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
                store: self.data_store.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn initialize_market_vault_ctx(
        &self,
        kind: TokenKind,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMarketVault<'info>> {
        let (mint, vault) = match kind {
            TokenKind::Market => (
                self.market_token_mint.to_account_info(),
                self.market_token_vault.to_account_info(),
            ),
            TokenKind::Long => (
                self.long_token_mint.to_account_info(),
                self.long_token_vault.to_account_info(),
            ),
            TokenKind::Short => (
                self.short_token_mint.to_account_info(),
                self.short_token_vault.to_account_info(),
            ),
        };
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeMarketVault {
                authority: self.authority.to_account_info(),
                store: self.data_store.to_account_info(),
                mint,
                vault,
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}

impl<'info> Authentication<'info> for CreateMarket<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> WithStore<'info> for CreateMarket<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.data_store.to_account_info()
    }
}
