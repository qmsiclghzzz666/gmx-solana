/// Keys.
pub mod keys;

use gmx_solana_utils::price::Decimal;

/// Market Token Mint Address Seed.
pub const MAREKT_TOKEN_MINT_SEED: &[u8] = b"market_token_mint";

/// Decimals of a market token.
pub const MARKET_TOKEN_DECIMALS: u8 = 9;

/// Market Vault Seed.
pub const MARKET_VAULT_SEED: &[u8] = b"market_vault";

/// Unit USD value i.e. `one`.
pub const MARKET_USD_UNIT: u128 = 10u128.pow(MARKET_DECIMALS as u32);

/// Adjustment factor for saving funding amount per size.
pub const FUNDING_AMOUNT_PER_SIZE_ADJUSTMENT: u128 = 10u128.pow((MARKET_DECIMALS >> 1) as u32);

/// USD value to amount divisor.
pub const MARKET_USD_TO_AMOUNT_DIVISOR: u128 =
    10u128.pow((MARKET_DECIMALS - MARKET_TOKEN_DECIMALS) as u32);

/// Deicmals of usd values of factors.
pub const MARKET_DECIMALS: u8 = Decimal::MAX_DECIMALS;
