use anchor_lang::prelude::*;

use crate::{
    constants::key::GLOBAL,
    states::{Amount, Config, DataStore, Factor, Roles, Seed},
    utils::internal,
};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = only_controller.bump,
    )]
    only_controller: Account<'info, Roles>,
    store: Account<'info, DataStore>,
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
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeConfig<'info> {
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

#[derive(Accounts)]
pub struct InsertAmount<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = only_controller.bump,
    )]
    only_controller: Account<'info, Roles>,
    store: Account<'info, DataStore>,
    #[account(
        mut,
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

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

#[derive(Accounts)]
pub struct InsertFactor<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = only_controller.bump,
    )]
    only_controller: Account<'info, Roles>,
    store: Account<'info, DataStore>,
    #[account(
        mut,
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

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}
