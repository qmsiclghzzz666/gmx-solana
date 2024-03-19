use anchor_lang::prelude::*;

use crate::{
    states::{DataStore, Nonce, NonceBytes, Roles, Seed},
    utils::internal,
};

/// Initialize a nonce account for the given data store.
pub fn initialize_nonce(ctx: Context<InitializeNonce>) -> Result<()> {
    ctx.accounts.nonce.init(ctx.bumps.nonce);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeNonce<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        space = 8 + Nonce::INIT_SPACE,
        payer = authority,
        seeds = [Nonce::SEED, store.key().as_ref()],
        bump,
    )]
    pub nonce: Account<'info, Nonce>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for InitializeNonce<'info> {
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

/// Increment the nonce value and return its bytes representation.
pub fn increment_nonce(ctx: Context<IncrementNonce>) -> Result<NonceBytes> {
    let nonce = &mut ctx.accounts.nonce;
    nonce.inc();
    Ok(nonce.nonce())
}

#[derive(Accounts)]
pub struct IncrementNonce<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        seeds = [Nonce::SEED, store.key().as_ref()],
        bump = nonce.bump,
    )]
    nonce: Account<'info, Nonce>,
}

impl<'info> internal::Authentication<'info> for IncrementNonce<'info> {
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

/// Get the nonce in bytes.
pub fn get_nonce_bytes(ctx: Context<GetNonceBytes>) -> Result<NonceBytes> {
    Ok(ctx.accounts.nonce.nonce())
}

#[derive(Accounts)]
pub struct GetNonceBytes<'info> {
    nonce: Account<'info, Nonce>,
}
