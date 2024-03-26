use anchor_lang::prelude::*;

use crate::{
    states::{
        deposit::{Receivers, TokenParams},
        DataStore, Deposit, Market, NonceBytes, Roles, Seed,
    },
    utils::internal,
};

/// Initialize a new [`Deposit`] account.
pub fn initialize_deposit(
    ctx: Context<InitializeDeposit>,
    nonce: NonceBytes,
    receivers: Receivers,
    tokens: TokenParams,
) -> Result<()> {
    ctx.accounts.deposit.init(
        ctx.bumps.deposit,
        &ctx.accounts.market,
        nonce,
        ctx.accounts.payer.key(),
        receivers,
        tokens,
    )
}

#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct InitializeDeposit<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        space = 8 + Deposit::INIT_SPACE,
        payer = payer,
        seeds = [Deposit::SEED, store.key().as_ref(), payer.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: Account<'info, Deposit>,
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for InitializeDeposit<'info> {
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

/// Remove deposit.
pub fn remove_deposit(_ctx: Context<RemoveDeposit>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveDeposit<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        close = user,
        has_one = user,
        seeds = [
            Deposit::SEED,
            store.key().as_ref(),
            user.key().as_ref(),
            &deposit.nonce,
        ],
        bump = deposit.bump,
    )]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports,
    /// and has been checked in `deposit`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
}

impl<'info> internal::Authentication<'info> for RemoveDeposit<'info> {
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
