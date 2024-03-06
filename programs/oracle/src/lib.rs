use anchor_lang::prelude::*;
use data_store::DataStore;
use gmx_solana_utils::to_seed;

/// Decimal type for storing prices.
pub mod decimal;

/// Price type.
pub mod price;

pub use self::{
    decimal::{Decimal, DecimalError},
    price::{Price, PriceMap},
};

declare_id!("8LmVjFpoR6hupp6WZZb6EbmupaXvivaCEk2iAHskr1en");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        // FIXME: Is it still correct if we not clear here?
        ctx.accounts.oracle.primary.clear();
        ctx.accounts.oracle.bump = ctx.bumps.oracle;
        ctx.accounts.oracle.role_store = *ctx.accounts.store.role_store();
        ctx.accounts.oracle.data_store = ctx.accounts.store.key();
        msg!("new oracle initialized with key: {}", key);
        Ok(())
    }

    pub fn get_price_from_feed(ctx: Context<GetPriceFromFeed>) -> Result<Round> {
        let round = chainlink_solana::latest_round_data(
            ctx.accounts.chainlink_program.to_account_info(),
            ctx.accounts.feed.to_account_info(),
        )?;
        msg!("answer: {}", round.answer);
        Round::try_new(round)
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + Oracle::INIT_SPACE,
        seeds = [Oracle::SEED, &store.key().to_bytes(), &to_seed(&key)],
        bump,
    )]
    pub oracle: Account<'info, Oracle>,
    pub system_program: Program<'info, System>,
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
pub struct GetPriceFromFeed<'info> {
    /// CHECK: it will be checked by chainlink.
    feed: UncheckedAccount<'info>,
    chainlink_program: Program<'info, Chainlink>,
}

/// Oracle Account.
#[account]
#[derive(InitSpace)]
pub struct Oracle {
    bump: u8,
    role_store: Pubkey,
    data_store: Pubkey,
    primary: PriceMap,
}

impl Oracle {
    /// Seed for PDA.
    pub const SEED: &'static [u8] = b"oracle";
}

/// The Chainlink Program.
pub struct Chainlink;

impl Id for Chainlink {
    fn id() -> Pubkey {
        chainlink_solana::ID
    }
}

/// Oracle Errors.
#[error_code]
pub enum OracleError {
    #[msg("Price of the given token already set")]
    PriceAlreadySet,
}
