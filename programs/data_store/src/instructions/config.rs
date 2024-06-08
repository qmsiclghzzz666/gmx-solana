use anchor_lang::prelude::*;

use crate::{
    constants::keys::GLOBAL,
    states::{Amount, Config, Factor, Seed, Store},
    utils::internal,
};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(
        init,
        payer = authority,
        space = 8 + Config::INIT_SPACE,
        seeds = [Config::SEED, store.key().as_ref()],
        bump,
    )]
    config: Account<'info, Config>,
    system_program: Program<'info, System>,
}

/// Initialize Config.
pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.bump = ctx.bumps.config;
    config.store = ctx.accounts.store.key();
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeConfig<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertAmount<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
}

/// Insert amount.
pub fn insert_amount(
    ctx: Context<InsertAmount>,
    key: &str,
    amount: Amount,
    new: bool,
) -> Result<()> {
    ctx.accounts
        .config
        .insert_amount(GLOBAL, key, amount, new)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertAmount<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertFactor<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
}

/// Insert factor.
pub fn insert_factor(
    ctx: Context<InsertFactor>,
    key: &str,
    factor: Factor,
    new: bool,
) -> Result<()> {
    ctx.accounts
        .config
        .insert_factor(GLOBAL, key, factor, new)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertFactor<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertAddress<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
}

/// Insert address.
pub fn insert_address(
    ctx: Context<InsertAddress>,
    key: &str,
    address: Pubkey,
    new: bool,
) -> Result<()> {
    ctx.accounts
        .config
        .insert_address(GLOBAL, key, &address, new)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertAddress<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
