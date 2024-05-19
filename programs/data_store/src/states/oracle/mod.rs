/// Price Map.
pub mod price_map;

/// Chainlink.
pub mod chainlink;

/// Pyth.
pub mod pyth;

/// Price Validator.
pub mod validator;

use core::fmt;

use crate::DataStoreError;
use anchor_lang::{prelude::*, Ids};
use num_enum::TryFromPrimitive;

use super::{Seed, TokenConfigMap};

pub use self::{
    chainlink::Chainlink,
    price_map::PriceMap,
    pyth::{Pyth, PythLegacy, PYTH_LEGACY_ID},
    validator::PriceValidator,
};

/// Oracle Account.
#[account]
#[derive(InitSpace, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Oracle {
    pub bump: u8,
    pub index: u8,
    pub primary: PriceMap,
    pub min_oracle_ts: i64,
    pub max_oracle_ts: i64,
}

impl Seed for Oracle {
    const SEED: &'static [u8] = b"oracle";
}

impl Oracle {
    /// Initialize the [`Oracle`].
    pub(crate) fn init(&mut self, bump: u8, index: u8) {
        self.clear_all_prices();
        self.bump = bump;
        self.index = index;
    }

    /// Set prices from remaining accounts.
    pub(crate) fn set_prices_from_remaining_accounts<'info>(
        &mut self,
        mut validator: PriceValidator,
        provider: &Interface<'info, PriceProvider>,
        map: &TokenConfigMap,
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
            let map = map.as_map();
            let token_config = map
                .get(token)
                .ok_or(DataStoreError::RequiredResourceNotFound)?;
            require!(token_config.enabled, DataStoreError::TokenConfigDisabled);
            require_eq!(token_config.expected_provider()?, *program.kind());
            let (oracle_ts, price) = match &program {
                PriceProviderProgram::Chainlink(program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    Chainlink::check_and_get_chainlink_price(
                        validator.clock(),
                        program,
                        token_config,
                        feed,
                    )?
                }
                PriceProviderProgram::Pyth(_program, kind) => {
                    let feed_id = token_config.get_feed(kind)?;
                    Pyth::check_and_get_price(validator.clock(), token_config, feed, &feed_id)?
                }
                PriceProviderProgram::PythLegacy(_program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    // We don't have to check the `feed_id` because the `feed` account is set by the token config keeper.
                    PythLegacy::check_and_get_price(validator.clock(), token_config, feed)?
                }
            };
            validator.validate_one(token_config, oracle_ts, &price)?;
            self.primary.set(token, price)?;
        }
        self.update_oracle_ts_range(validator)?;
        Ok(())
    }

    fn update_oracle_ts_range(&mut self, mut validator: PriceValidator) -> Result<()> {
        validator.merge_range(self.min_oracle_ts, self.max_oracle_ts);
        (self.min_oracle_ts, self.max_oracle_ts) = validator.finish()?;
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear_all_prices(&mut self) {
        self.primary.clear();
        self.min_oracle_ts = i64::MAX;
        self.max_oracle_ts = i64::MIN;
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
