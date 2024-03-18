use anchor_lang::prelude::*;

use crate::{
    states::{
        deposit::{Receivers, Tokens},
        DataStore, Deposit, NonceBytes, Roles, Seed,
    },
    utils::internal,
};

/// Initialize a new [`Deposit`] account.
pub fn initialize_deposit(
    ctx: Context<InitializeDeposit>,
    nonce: NonceBytes,
    market: Pubkey,
    receivers: Receivers,
    tokens: Tokens,
) -> Result<()> {
    ctx.accounts.deposit.init(
        ctx.bumps.deposit,
        nonce,
        ctx.accounts.payer.key(),
        market,
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
