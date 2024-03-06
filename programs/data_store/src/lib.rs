use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;
use role_store::{Authenticate, Authorization, Role, RoleStore};

declare_id!("8hJ2dGQ2Ccr5G6iEqQQEoBApRSXt7Jn8Qyf9Qf3eLBX2");

#[program]
pub mod data_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        ctx.accounts
            .data_store
            .init(ctx.accounts.role_store.key(), &key, ctx.bumps.data_store);
        Ok(())
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn set_address(ctx: Context<SetAddress>, _key: String, value: Pubkey) -> Result<()> {
        ctx.accounts.address.value = value;
        ctx.accounts.address.bump = ctx.bumps.address;
        Ok(())
    }

    pub fn get_address(ctx: Context<GetAddress>, _key: String) -> Result<Pubkey> {
        Ok(ctx.accounts.address.value)
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub role_store: Account<'info, RoleStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + DataStore::INIT_SPACE,
        seeds = [DataStore::SEED, &role_store.key().to_bytes(), &to_seed(&key)],
        bump,
    )]
    pub data_store: Account<'info, DataStore>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct DataStore {
    role_store: Pubkey,
    #[max_len(32)]
    key: Vec<u8>,
    bump: u8,
}

impl DataStore {
    /// Seed.
    pub const SEED: &'static [u8] = b"data_store";

    fn init(&mut self, role_store: Pubkey, key: &str, bump: u8) {
        self.role_store = role_store;
        self.key = to_seed(key).into();
        self.bump = bump;
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct SetAddress<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Role>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Address::INIT_SPACE,
        seeds = [Address::SEED, &store.key().to_bytes(), &to_seed(&key)],
        bump,
    )]
    pub address: Account<'info, Address>,
    pub system_program: Program<'info, System>,
}

impl<'info> Authorization<'info> for SetAddress<'info> {
    fn store(&self) -> Pubkey {
        self.store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_controller
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct GetAddress<'info> {
    pub store: Account<'info, DataStore>,
    #[account(
        seeds = [Address::SEED, &store.key().to_bytes(), &to_seed(&key)],
        bump = address.bump,
    )]
    pub address: Account<'info, Address>,
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
    #[msg("Mismatched role store")]
    MismatchedRoleStore,
}
