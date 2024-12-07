use anchor_lang::prelude::*;

declare_id!("GTtRSYha5h8S26kPFHgYKUf8enEgabkTFwW7UToXAHoY");

#[program]
pub mod gmsol_treasury {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
