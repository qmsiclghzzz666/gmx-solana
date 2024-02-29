use anchor_lang::prelude::*;

declare_id!("2mzvgXHKwkGTQpBWxDD7ebbHK4V1UpobmoCCFd9eQ4sa");

#[program]
pub mod gmx_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
