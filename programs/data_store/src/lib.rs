use anchor_lang::prelude::*;
use role_store::membership::Membership;

declare_id!("8hJ2dGQ2Ccr5G6iEqQQEoBApRSXt7Jn8Qyf9Qf3eLBX2");

#[program]
pub mod data_store {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    authority: Signer<'info>,
    #[account(
        has_one = authority,
        constraint = membership.is_admin(),
    )]
    membership: Account<'info, Membership>,
}
