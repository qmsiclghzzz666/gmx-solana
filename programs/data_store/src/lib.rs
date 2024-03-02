use anchor_lang::prelude::*;
use role_store::membership::Membership;

declare_id!("8hJ2dGQ2Ccr5G6iEqQQEoBApRSXt7Jn8Qyf9Qf3eLBX2");

#[program]
pub mod data_store {
    use super::*;

    pub fn set_address(ctx: Context<SetAddress>, key: String, value: Pubkey) -> Result<()> {
        require_gte!(64, key.len(), DataStoreError::KeyTooLong);
        ctx.accounts.address.value = value;
        ctx.accounts.address.bump = ctx.bumps.address;
        Ok(())
    }

    pub fn get_address(ctx: Context<GetAddress>, key: String) -> Result<Pubkey> {
        require_gte!(64, key.len(), DataStoreError::KeyTooLong);
        Ok(ctx.accounts.address.value)
    }
}

/// Convert a string to a seed.
pub fn to_seed(key: &str) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;
    hash(key.as_bytes()).to_bytes()
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct SetAddress<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        has_one = authority,
        constraint = only_controller.is_controller(),
    )]
    only_controller: Account<'info, Membership>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Address::INIT_SPACE,
        seeds = [Address::SEED, &to_seed(&key)],
        bump,
    )]
    address: Account<'info, Address>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct GetAddress<'info> {
    #[account(
        seeds = [Address::SEED, &to_seed(&key)],
        bump = address.bump,
    )]
    address: Account<'info, Address>,
}

#[account]
#[derive(InitSpace)]
pub struct Address {
    value: Pubkey,
    bump: u8,
}

impl Address {
    /// Seed for [`Address`]
    pub const SEED: &'static [u8] = b"address";
}

#[error_code]
pub enum DataStoreError {
    #[msg("the len of key in bytes cannot be greater than 64")]
    KeyTooLong,
}
