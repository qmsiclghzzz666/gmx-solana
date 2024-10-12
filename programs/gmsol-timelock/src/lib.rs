use anchor_lang::prelude::*;

declare_id!("timedreYasWZUyAgofdmjFVJwk3LKZZq6QJtgpc1aqv");

#[program]
pub mod gmsol_timelock {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
