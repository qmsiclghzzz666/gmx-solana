/// Price Map.
pub mod price_map;

/// Chainlink.
pub mod chainlink;

/// Pyth.
pub mod pyth;

/// Price Validator.
pub mod validator;

/// Oracle time validation.
pub mod time;

use core::fmt;

use crate::{states::TokenMapAccess, DataStoreError, StoreResult};
use anchor_lang::{prelude::*, Ids};
use num_enum::TryFromPrimitive;

use super::{HasMarketMeta, Seed, TokenMapRef};

pub use self::{
    chainlink::Chainlink,
    price_map::PriceMap,
    pyth::{Pyth, PythLegacy, PYTH_LEGACY_ID},
    time::{ValidateOracleTime, ValidateOracleTimeExt},
    validator::PriceValidator,
};

/// Oracle Account.
#[account]
#[derive(InitSpace, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Oracle {
    pub bump: u8,
    pub store: Pubkey,
    pub index: u8,
    pub primary: PriceMap,
    pub min_oracle_ts: i64,
    pub max_oracle_ts: i64,
    pub min_oracle_slot: Option<u64>,
}

impl Seed for Oracle {
    const SEED: &'static [u8] = b"oracle";
}

impl Oracle {
    /// Initialize the [`Oracle`].
    pub(crate) fn init(&mut self, bump: u8, store: Pubkey, index: u8) {
        self.clear_all_prices();
        self.bump = bump;
        self.store = store;
        self.index = index;
    }

    /// Set prices from remaining accounts.
    pub(crate) fn set_prices_from_remaining_accounts<'info>(
        &mut self,
        mut validator: PriceValidator,
        provider: &Interface<'info, PriceProvider>,
        map: &TokenMapRef,
        tokens: &[Pubkey],
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        require!(self.primary.is_empty(), DataStoreError::PricesAlreadySet);
        require!(
            tokens.len() <= PriceMap::MAX_TOKENS,
            DataStoreError::ExceedMaxLengthLimit
        );
        require!(
            tokens.len() <= remaining_accounts.len(),
            ErrorCode::AccountNotEnoughKeys
        );
        let program = PriceProviderProgram::from_interface(provider);
        // Assume the remaining accounts are arranged in the following way:
        // [token_config, feed; tokens.len()] [..remaining]
        for (idx, token) in tokens.iter().enumerate() {
            let feed = &remaining_accounts[idx];
            let token_config = map
                .get(token)
                .ok_or(DataStoreError::RequiredResourceNotFound)?;
            require!(
                token_config.is_enabled(),
                DataStoreError::TokenConfigDisabled
            );
            require_eq!(token_config.expected_provider()?, *program.kind());
            let (oracle_slot, oracle_ts, price, kind) = match &program {
                PriceProviderProgram::Chainlink(program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    let (oracle_slot, oracle_ts, price) = Chainlink::check_and_get_chainlink_price(
                        validator.clock(),
                        program,
                        token_config,
                        feed,
                    )?;
                    (oracle_slot, oracle_ts, price, kind)
                }
                PriceProviderProgram::Pyth(_program, kind) => {
                    let feed_id = token_config.get_feed(kind)?;
                    let (oracle_slot, oracle_ts, price) =
                        Pyth::check_and_get_price(validator.clock(), token_config, feed, &feed_id)?;
                    (oracle_slot, oracle_ts, price, kind)
                }
                PriceProviderProgram::PythLegacy(_program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    // We don't have to check the `feed_id` because the `feed` account is set by the token config keeper.
                    let (oracle_slot, oracle_ts, price) =
                        PythLegacy::check_and_get_price(validator.clock(), token_config, feed)?;
                    (oracle_slot, oracle_ts, price, kind)
                }
            };
            validator.validate_one(token_config, kind, oracle_ts, oracle_slot, &price)?;
            self.primary.set(token, price)?;
        }
        self.update_oracle_ts_and_slot(validator)?;
        Ok(())
    }

    fn update_oracle_ts_and_slot(&mut self, mut validator: PriceValidator) -> Result<()> {
        validator.merge_range(self.min_oracle_slot, self.min_oracle_ts, self.max_oracle_ts);
        if let Some((min_slot, min_ts, max_ts)) = validator.finish()? {
            self.min_oracle_slot = Some(min_slot);
            self.min_oracle_ts = min_ts;
            self.max_oracle_ts = max_ts;
        }
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear_all_prices(&mut self) {
        self.primary.clear();
        self.min_oracle_ts = i64::MAX;
        self.max_oracle_ts = i64::MIN;
        self.min_oracle_slot = None;
    }

    /// Validate oracle time.
    pub(crate) fn validate_time(&self, target: &impl ValidateOracleTime) -> StoreResult<()> {
        if self.max_oracle_ts < self.min_oracle_ts {
            msg!("min = {}, max = {}", self.min_oracle_ts, self.max_oracle_ts);
            return Err(DataStoreError::InvalidOracleTsTrange);
        }
        target.validate_min_oracle_slot(self)?;
        target.validate_min_oracle_ts(self)?;
        target.validate_max_oracle_ts(self)?;
        Ok(())
    }

    /// Get prices for the market
    pub(crate) fn market_prices(
        &self,
        market: &impl HasMarketMeta,
    ) -> Result<gmx_core::action::Prices<u128>> {
        let meta = market.market_meta();
        let prices = gmx_core::action::Prices {
            index_token_price: self
                .primary
                .get(&meta.index_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
            long_token_price: self
                .primary
                .get(&meta.long_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
            short_token_price: self
                .primary
                .get(&meta.short_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
        };
        Ok(prices)
    }
}

/// Price Provider.
pub struct PriceProvider;

pub(crate) static PRICE_PROVIDER_IDS: [Pubkey; 3] = [
    pyth_solana_receiver_sdk::ID,
    chainlink_solana::ID,
    PYTH_LEGACY_ID,
];

impl Ids for PriceProvider {
    fn ids() -> &'static [Pubkey] {
        &PRICE_PROVIDER_IDS
    }
}

/// Supported Price Provider Kind.
#[repr(u8)]
#[derive(Clone, Copy, Default, TryFromPrimitive, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
pub enum PriceProviderKind {
    /// Pyth Oracle V2.
    #[default]
    Pyth = 0,
    /// Chainlink Data Feed.
    Chainlink = 1,
    /// Legacy Pyth Push Oracle.
    PythLegacy = 2,
}

impl fmt::Display for PriceProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pyth => write!(f, "Pyth"),
            Self::Chainlink => write!(f, "Chainlink"),
            Self::PythLegacy => write!(f, "PythLegacy"),
        }
    }
}

#[cfg(feature = "utils")]
impl PriceProviderKind {
    /// Get correspoding program address.
    pub fn program(&self) -> Pubkey {
        match self {
            Self::Pyth => Pyth::id(),
            Self::Chainlink => Chainlink::id(),
            Self::PythLegacy => PythLegacy::id(),
        }
    }
}

/// Supported Price Provider Programs.
/// The [`PriceProviderKind`] field is used as index
/// to query the correspoding feed from the token config.
enum PriceProviderProgram<'info> {
    Chainlink(AccountInfo<'info>, PriceProviderKind),
    Pyth(AccountInfo<'info>, PriceProviderKind),
    PythLegacy(AccountInfo<'info>, PriceProviderKind),
}

impl<'info> PriceProviderProgram<'info> {
    fn kind(&self) -> &PriceProviderKind {
        match self {
            Self::Chainlink(_, kind) | Self::Pyth(_, kind) | Self::PythLegacy(_, kind) => kind,
        }
    }
}

impl<'info> PriceProviderProgram<'info> {
    fn from_interface(interface: &Interface<'info, PriceProvider>) -> Self {
        if *interface.key == Chainlink::id() {
            Self::Chainlink(interface.to_account_info(), PriceProviderKind::Chainlink)
        } else if *interface.key == Pyth::id() {
            Self::Pyth(interface.to_account_info(), PriceProviderKind::Pyth)
        } else if *interface.key == PythLegacy::id() {
            Self::PythLegacy(interface.to_account_info(), PriceProviderKind::PythLegacy)
        } else {
            unreachable!();
        }
    }
}
