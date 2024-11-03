use anchor_lang::prelude::*;

declare_id!("4nMxSRfeW7W2zFbN8FJ4YDvuTzEzCo1e6GzJxJLnDUoZ");

#[program]
pub mod mock_chainlink_verifier {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.verifier_account.authority = ctx.accounts.payer.key();
        Ok(())
    }

    pub fn verify(_ctx: Context<VerifyContext>, _signed_report: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub fn verify_bulk(_ctx: Context<VerifyContext>, _signed_reports: Vec<Vec<u8>>) -> Result<()> {
        Ok(())
    }
}

pub const DEFAULT_VERIFIER_ACCOUNT_SEEDS: &[u8; 8] = b"verifier";

/// Verifyer Account.
#[account]
#[derive(InitSpace)]
pub struct VerifierAccount {
    authority: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Verifier Account.
    #[account(
        init,
        payer = payer,
        space = 8 + VerifierAccount::INIT_SPACE,
        seeds = [DEFAULT_VERIFIER_ACCOUNT_SEEDS],
        bump,
    )]
    pub verifier_account: Account<'info, VerifierAccount>,
    /// The System Program.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyContext<'info> {
    /// Verifier Account.
    #[account(
        seeds = [DEFAULT_VERIFIER_ACCOUNT_SEEDS],
        bump,
    )]
    pub verifier_account: Account<'info, VerifierAccount>,
}
