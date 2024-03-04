use anchor_lang::prelude::*;

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<Round> {
        let round = chainlink_solana::latest_round_data(
            ctx.accounts.chainlink.to_account_info(),
            ctx.accounts.feed.to_account_info(),
        )?;
        msg!("answer: {}", round.answer);
        Round::try_new(round)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Round {
    pub round_id: u32,
    pub slot: u64,
    pub timestamp: u32,
    pub answer: i128,
    pub sys_timestamp: i64,
}

impl Round {
    fn try_new(round: chainlink_solana::Round) -> Result<Self> {
        let clock = Clock::get()?;
        let chainlink_solana::Round {
            round_id,
            slot,
            timestamp,
            answer,
        } = round;
        Ok(Self {
            round_id,
            slot,
            timestamp,
            answer,
            sys_timestamp: clock.unix_timestamp,
        })
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: it will be checked by chainlink.
    feed: UncheckedAccount<'info>,
    /// CHECK: it will be checked by chainlink.
    chainlink: UncheckedAccount<'info>,
}
