use anchor_lang::prelude::*;

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
