use anchor_lang::prelude::*;

declare_id!("BGDJg2u2NWwUE5q4Q4masGCFBVAhJ5pKrMbVSwjVwo8m");

#[program]
pub mod gmsol_liquidity_provider {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    payer: Signer<'info>,
}
