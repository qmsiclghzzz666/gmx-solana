use anchor_lang::prelude::*;

declare_id!("trekjLtMLpfJHVxBVhng99koG3LPAxzTBLMYco9vfJQ");

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
