use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{
    constants,
    states::{DataStore, Roles},
    utils::internal,
};

/// Initialize a new market token.
#[allow(unused_variables)]
pub fn initialize_market_token(
    ctx: Context<InitializeMarketToken>,
    index_token_mint: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token_mint: Pubkey, long_token_mint: Pubkey, short_token_mint: Pubkey)]
pub struct InitializeMarketToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        mint::decimals = constants::MARKET_TOKEN_DECIMALS,
        // We directly use the store as the authority.
        mint::authority = store.key(),
        seeds = [
            constants::MAREKT_TOKEN_MINT_SEED,
            store.key().as_ref(),
            index_token_mint.as_ref(),
            long_token_mint.key().as_ref(),
            short_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub market_token_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for InitializeMarketToken<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_market_keeper
    }
}

/// Mint the given amount of market tokens to the destination account.
pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
    anchor_spl::token::mint_to(
        ctx.accounts
            .mint_to_ctx()
            .with_signer(&[&ctx.accounts.store.pda_seeds()]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MintMarketTokenTo<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = market_token_mint)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for MintMarketTokenTo<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

impl<'info> MintMarketTokenTo<'info> {
    fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.market_token_mint.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.store.to_account_info(),
            },
        )
    }
}

/// Initialize a vault of the given token for a market.
/// The address is derived from token mint addresses (the `market_token_mint` seed is optional).
#[allow(unused_variables)]
pub fn initialize_market_vault(
    ctx: Context<InitializeMarketVault>,
    market_token_mint: Option<Pubkey>,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_token_mint: Option<Pubkey>)]
pub struct InitializeMarketVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        // We use the store as the authority of the token account.
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            mint.key().as_ref(),
            market_token_mint.as_ref().map(|key| key.as_ref()).unwrap_or(&[]),
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for InitializeMarketVault<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_market_keeper
    }
}

/// Transfer the given amount of tokens out to the destination account.
pub fn market_vault_transfer_out(ctx: Context<MarketVaultTransferOut>, amount: u64) -> Result<()> {
    anchor_spl::token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&ctx.accounts.store.pda_seeds()]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MarketVaultTransferOut<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    // FIXME: this is a bug to not checking the store.
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market_vault.mint)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for MarketVaultTransferOut<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

impl<'info> MarketVaultTransferOut<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.market_vault.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.store.to_account_info(),
            },
        )
    }
}
