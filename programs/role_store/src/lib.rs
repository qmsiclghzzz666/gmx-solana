use anchor_lang::prelude::*;

/// Membership.
pub mod membership;

pub use self::membership::Membership;

declare_id!("KLAh3o2hMDJT26zBxv6RYrxcUxBjPCEeSt3Q1rS4JMU");

#[program]
pub mod role_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts
            .admin_membership
            .grant_role(Membership::ROLE_ADMIN, ctx.bumps.admin_membership)
    }

    pub fn grant_role(ctx: Context<GrantRole>, role_key: String) -> Result<()> {
        ctx.accounts
            .membership
            .grant_role(&role_key, ctx.bumps.membership)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Membership::MAX_SIZE,
        seeds = [Membership::SEED, admin.key().as_ref()],
        bump,
    )]
    pub admin_membership: Account<'info, Membership>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GrantRole<'info> {
    pub permission: OnlyRoleAdmin<'info>,
    /// CHECK: We only use it as a pubkey.
    pub member: UncheckedAccount<'info>,
    #[account(
        init,
        payer = permission.authority.signer,
        space = 8 + Membership::MAX_SIZE,
        seeds = [Membership::SEED, member.key().as_ref()],
        bump,
    )]
    pub membership: Account<'info, Membership>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Member<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [Membership::SEED, signer.key().as_ref()],
        bump = membership.bump(),
    )]
    pub membership: Account<'info, Membership>,
}

#[derive(Accounts)]
pub struct OnlyRoleAdmin<'info> {
    #[account(constraint = authority.membership.is_admin() @ RoleStoreError::InvalidRole)]
    pub authority: Member<'info>,
}

#[error_code]
pub enum RoleStoreError {
    #[msg("Invalid role name")]
    InvalidRoleName,
    #[msg("Invalid role")]
    InvalidRole,
}
