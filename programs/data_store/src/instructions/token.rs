use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{constants, states::Store, utils::internal};

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
    pub store: AccountLoader<'info, Store>,
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

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// Mint the given amount of market tokens to the destination account.
pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
    anchor_spl::token::mint_to(
        ctx.accounts
            .mint_to_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.pda_seeds()]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MintMarketTokenTo<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
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

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
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

/// Burn the given amount of market tokens from the given account.
///
/// ## Notes
/// - The `from` account is expected to be owned by `store`.
pub fn burn_market_token_from(ctx: Context<BurnMarketTokenFrom>, amount: u64) -> Result<()> {
    anchor_spl::token::burn(
        ctx.accounts
            .burn_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.pda_seeds()]),
        amount,
    )
}

#[derive(Accounts)]
pub struct BurnMarketTokenFrom<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = market_token_mint)]
    pub from: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for BurnMarketTokenFrom<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> BurnMarketTokenFrom<'info> {
    fn burn_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.market_token_mint.to_account_info(),
                from: self.from.to_account_info(),
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
    pub store: AccountLoader<'info, Store>,
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

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// Transfer the given amount of tokens out to the destination account.
pub fn market_vault_transfer_out(ctx: Context<MarketVaultTransferOut>, amount: u64) -> Result<()> {
    anchor_spl::token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.pda_seeds()]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MarketVaultTransferOut<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
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

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
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

#[derive(Accounts)]
#[instruction(timestamp: i64)]
pub struct UseClaimableAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub mint: Account<'info, Mint>,
    /// CHECK: check by CPI.
    pub user: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = authority,
        token::mint = mint,
        // We use the store as the authority of the token account.
        token::authority = store,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            mint.key().as_ref(),
            user.key().as_ref(),
            &store.load()?.claimable_time_key(timestamp)?,
        ],
        bump,
    )]
    pub account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Prepare claimable account.
pub fn use_claimable_account(
    ctx: Context<UseClaimableAccount>,
    _timestamp: i64,
    amount: u64,
) -> Result<()> {
    if ctx.accounts.account.delegate.is_none() || ctx.accounts.account.delegated_amount != amount {
        anchor_spl::token::approve(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Approve {
                    to: ctx.accounts.account.to_account_info(),
                    delegate: ctx.accounts.user.to_account_info(),
                    authority: ctx.accounts.store.to_account_info(),
                },
                &[&ctx.accounts.store.load()?.pda_seeds()],
            ),
            0,
        )?;
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for UseClaimableAccount<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
#[instruction(user: Pubkey, timestamp: i64)]
pub struct CloseEmptyClaimableAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        // We use the store as the authority of the token account.
        token::authority = store,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            mint.key().as_ref(),
            user.key().as_ref(),
            &store.load()?.claimable_time_key(timestamp)?,
        ],
        bump,
    )]
    pub account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Close claimable account if it is empty.
pub fn close_empty_claimable_account(
    ctx: Context<CloseEmptyClaimableAccount>,
    _user: Pubkey,
    _timestamp: i64,
) -> Result<()> {
    if ctx.accounts.account.amount == 0 {
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.account.to_account_info(),
                destination: ctx.accounts.authority.to_account_info(),
                authority: ctx.accounts.store.to_account_info(),
            },
            &[&ctx.accounts.store.load()?.pda_seeds()],
        ))?;
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for CloseEmptyClaimableAccount<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
