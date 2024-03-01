use anchor_lang::prelude::*;

declare_id!("HLkiY9JScepfVa8UJ9dfy3gnKfQhWvFZ4iK8hANxuTCy");

#[program]
pub mod data_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
