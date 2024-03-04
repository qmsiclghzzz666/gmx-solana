use anchor_lang::prelude::*;

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<i128> {
        let round = chainlink_solana::latest_round_data(
            ctx.accounts.chainlink.to_account_info(),
            ctx.accounts.feed.to_account_info(),
        )?;
        msg!("answer: {}", round.answer);
        Ok(round.answer)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: it will be checked by chainlink.
    feed: UncheckedAccount<'info>,
    /// CHECK: it will be checked by chainlink.
    chainlink: UncheckedAccount<'info>,
}
