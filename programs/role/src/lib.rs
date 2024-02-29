use anchor_lang::prelude::*;

declare_id!("EUE8AaF2X5Z7Pbi3j3DU3zquhe9QpuANqZWLvDQM1vJ");

#[program]
pub mod role {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
