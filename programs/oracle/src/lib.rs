use anchor_lang::prelude::*;
use data_store::DataStore;
use gmx_solana_utils::to_seed;
use role_store::{Authenticate, Authorization, Role};

/// Decimal type for storing prices.
pub mod decimal;

/// Price type.
pub mod price;

/// Utils.
pub mod utils;

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

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        require!(
            ctx.accounts.oracle.primary.is_empty(),
            OracleError::PricesAlreadySet
        );
        require!(
            tokens.len() <= PriceMap::MAX_TOKENS,
            OracleError::ExceedMaxTokens
        );
        // We are going to parse the remaining accounts to address accounts and feed accounts in order.
        // It won't overflow since we has checked the length before.
        let remaining = ctx.remaining_accounts;
        require!(
            (tokens.len() << 1) <= remaining.len(),
            OracleError::NotEnoughAccountInfos
        );
        // Assume the remaining accounts are arranged in the following way:
        // [address, feed; tokens.len()] [..remaining]
        for (idx, token) in tokens.iter().enumerate() {
            let address_idx = idx << 1;
            let feed_idx = address_idx + 1;
            let price = utils::check_and_get_chainlink_price(
                &ctx.accounts.chainlink_program,
                &ctx.accounts.store,
                &remaining[address_idx],
                &remaining[feed_idx],
                token,
            )?;
            ctx.accounts.oracle.primary.set(token, price)?;
        }
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
        seeds = [Oracle::SEED, store.key().as_ref(), &to_seed(&key)],
        bump,
    )]
    pub oracle: Account<'info, Oracle>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPricesFromPriceFeed<'info> {
    pub authority: Signer<'info>,
    pub role: Account<'info, Role>,
    pub store: Account<'info, DataStore>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    pub chainlink_program: Program<'info, Chainlink>,
}

impl<'info> Authorization<'info> for SetPricesFromPriceFeed<'info> {
    fn role_store(&self) -> Pubkey {
        *self.store.role_store()
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.role
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
    #[msg("Prices already set")]
    PricesAlreadySet,
    #[msg("Exceed the maximum number of tokens")]
    ExceedMaxTokens,
    #[msg("Not enough account infos")]
    NotEnoughAccountInfos,
    #[msg("Invalid token config account")]
    InvalidTokenConfigAccount,
    #[msg("Invalid price feed account")]
    InvalidPriceFeedAccount,
    #[msg("Invalid price from data feed")]
    InvalidDataFeedPrice,
}
