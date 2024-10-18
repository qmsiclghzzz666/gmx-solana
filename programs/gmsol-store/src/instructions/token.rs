use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer},
    token_interface,
};

use crate::{
    constants,
    states::Store,
    utils::{internal, token::must_be_uninitialized},
};

/// The accounts definition for [`mint_market_token_to`](crate::gmsol_store::mint_market_token_to).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::mint_market_token_to)*
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

/// Mint the given amount of market tokens to the destination account.
///
/// ## CHECK
/// - Only CONTROLLER can mint market token.
pub(crate) fn unchecked_mint_market_token_to(
    ctx: Context<MintMarketTokenTo>,
    amount: u64,
) -> Result<()> {
    anchor_spl::token::mint_to(
        ctx.accounts
            .mint_to_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.signer_seeds()]),
        amount,
    )
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

/// The accounts definition for [`burn_market_token_from`](crate::gmsol_store::burn_market_token_from).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::burn_market_token_from)*
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

/// Burn the given amount of market tokens from the given account.
///
/// ## CHECK
/// - Only CONTROLLER can burn market tokens.
///
/// ## Notes
/// - The `from` account is expected to be owned by `store`.
pub(crate) fn unchecked_burn_market_token_from(
    ctx: Context<BurnMarketTokenFrom>,
    amount: u64,
) -> Result<()> {
    anchor_spl::token::burn(
        ctx.accounts
            .burn_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.signer_seeds()]),
        amount,
    )
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

/// The accounts definition for [`initialize_market_vault`](crate::gmsol_store::initialize_market_vault).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::initialize_market_vault)*
#[derive(Accounts)]
#[instruction(market_token_mint: Option<Pubkey>)]
pub struct InitializeMarketVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
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

/// Initialize a vault of the given token for a market.
/// The address is derived from token mint addresses (the `market_token_mint` seed is optional).
///
/// ## CHECK
/// - Only MARKET_KEEPER can initialize market vault.
#[allow(unused_variables)]
pub(crate) fn unchecked_initialize_market_vault(
    ctx: Context<InitializeMarketVault>,
    market_token_mint: Option<Pubkey>,
) -> Result<()> {
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeMarketVault<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`market_vault_transfer_out`](crate::gmsol_store::market_vault_transfer_out).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::market_vault_transfer_out)*
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

/// Transfer the given amount of tokens out to the destination account.
///
/// ## CHECK
/// - Only CONTROLLER can transfer out from market vault.
pub(crate) fn unchecked_market_vault_transfer_out(
    ctx: Context<MarketVaultTransferOut>,
    amount: u64,
) -> Result<()> {
    anchor_spl::token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&ctx.accounts.store.load()?.signer_seeds()]),
        amount,
    )
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

/// The accounts definition for [`use_claimable_account`](crate::gmsol_store::use_claimable_account).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::use_claimable_account)*
#[derive(Accounts)]
#[instruction(timestamp: i64)]
pub struct UseClaimableAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub mint: Account<'info, Mint>,
    /// CHECK: check by CPI.
    pub owner: UncheckedAccount<'info>,
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
            owner.key().as_ref(),
            &store.load()?.claimable_time_key(timestamp)?,
        ],
        bump,
    )]
    pub account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Prepare claimable account.
///
/// ## CHECK
/// - Only ORDER_KEEPER can use claimable account.
pub(crate) fn unchecked_use_claimable_account(
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
                    delegate: ctx.accounts.owner.to_account_info(),
                    authority: ctx.accounts.store.to_account_info(),
                },
                &[&ctx.accounts.store.load()?.signer_seeds()],
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

/// The accounts definition for [`close_empty_claimable_account`](crate::gmsol_store::close_empty_claimable_account).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::close_empty_claimable_account)*
#[derive(Accounts)]
#[instruction(timestamp: i64)]
pub struct CloseEmptyClaimableAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub mint: Account<'info, Mint>,
    /// CHECK: only use to reference the owner.
    pub owner: UncheckedAccount<'info>,
    /// CHECK: will be checked during the execution.
    #[account(
        mut,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            mint.key().as_ref(),
            owner.key().as_ref(),
            &store.load()?.claimable_time_key(timestamp)?,
        ],
        bump,
    )]
    pub account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Close claimable account if it is empty.
///
/// ## CHECK
/// - Only ORDER_KEEPER can close claimable account.
pub(crate) fn unchecked_close_empty_claimable_account(
    ctx: Context<CloseEmptyClaimableAccount>,
    _timestamp: i64,
) -> Result<()> {
    if must_be_uninitialized(&ctx.accounts.account) {
        return Ok(());
    }
    let account = ctx.accounts.account.to_account_info();
    let amount = anchor_spl::token::accessor::amount(&account)?;
    if amount == 0 {
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.account.to_account_info(),
                destination: ctx.accounts.authority.to_account_info(),
                authority: ctx.accounts.store.to_account_info(),
            },
            &[&ctx.accounts.store.load()?.signer_seeds()],
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

/// The accounts definition for [`prepare_associated_token_account`](crate::gmsol_store::prepare_associated_token_account).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::prepare_associated_token_account)*
#[derive(Accounts)]
pub struct PrepareAssociatedTokenAccount<'info> {
    /// The payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: only use as the owner of the token account.
    pub owner: UncheckedAccount<'info>,
    /// The mint account for the token account.
    pub mint: InterfaceAccount<'info, token_interface::Mint>,
    /// The token account to prepare.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub account: InterfaceAccount<'info, token_interface::TokenAccount>,
    /// The [`System`] program.
    pub system_program: Program<'info, System>,
    /// The [`Token`] program.
    pub token_program: Interface<'info, token_interface::TokenInterface>,
    /// The [`AssociatedToken`] program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_associated_token_account(
    _ctx: Context<PrepareAssociatedTokenAccount>,
) -> Result<()> {
    Ok(())
}
