use anchor_lang::prelude::*;

/// Membership.
pub mod membership;

pub use self::membership::Membership;

declare_id!("H7L3aYUzR3joc6Heyonjt1thpWdtwYcTVQvCrB2xDsdi");

#[program]
pub mod role_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.admin_membership.grant_role(
            Membership::ROLE_ADMIN,
            ctx.bumps.admin_membership,
            ctx.accounts.admin.key(),
        )
    }

    pub fn grant_role(ctx: Context<GrantRole>, role_key: String) -> Result<()> {
        ctx.accounts.membership.grant_role(
            &role_key,
            ctx.bumps.membership,
            ctx.accounts.member.key(),
        )
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Membership::INIT_SPACE,
        seeds = [Membership::SEED, Membership::ROLE_ADMIN.as_bytes(), admin.key().as_ref()],
        bump,
    )]
    pub admin_membership: Account<'info, Membership>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(role_key: String)]
pub struct GrantRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        has_one = authority,
        constraint = only_admin.is_admin() @ RoleStoreError::InvalidRole,
    )]
    pub only_admin: Account<'info, Membership>,
    /// CHECK: We only use it as a pubkey.
    pub member: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Membership::INIT_SPACE,
        seeds = [Membership::SEED, role_key.as_bytes(), member.key().as_ref()],
        bump,
    )]
    pub membership: Account<'info, Membership>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum RoleStoreError {
    #[msg("Invalid role name")]
    InvalidRoleName,
    #[msg("Invalid role")]
    InvalidRole,
}
