use core::fmt;

use anchor_lang::{prelude::*, Ids, Owners};
use dual_vec_map::DualVecMap;
use gmx_solana_utils::price::{Decimal, Price};
use num_enum::TryFromPrimitive;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::DataStoreError;

use super::{Seed, TokenConfig, TokenConfigMap};

/// Maximum number of tokens for a single `Price Map` to store.
const MAX_TOKENS: usize = 32;

/// Price Map.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Default)]
pub struct PriceMap {
    #[max_len(MAX_TOKENS)]
    prices: Vec<Price>,
    #[max_len(MAX_TOKENS)]
    tokens: Vec<Pubkey>,
}

impl PriceMap {
    /// Maximum number of tokens for a single `Price Map` to store.
    pub const MAX_TOKENS: usize = MAX_TOKENS;

    fn as_map(&self) -> DualVecMap<&Vec<Pubkey>, &Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&self.tokens, &self.prices)
    }

    fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<Pubkey>, &mut Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&mut self.tokens, &mut self.prices)
    }

    /// Get price of the given token key.
    pub fn get(&self, token: &Pubkey) -> Option<Price> {
        self.as_map().get(token).copied()
    }

    /// Set the price of the given token.
    /// # Error
    /// Return error if it already set.
    pub(crate) fn set(&mut self, token: &Pubkey, price: Price) -> Result<()> {
        self.as_map_mut()
            .try_insert(*token, price)
            .map_err(|_| DataStoreError::PriceAlreadySet)?;
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear(&mut self) {
        self.tokens.clear();
        self.prices.clear();
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Oracle Account.
#[account]
#[derive(InitSpace, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Oracle {
    pub bump: u8,
    pub index: u8,
    pub primary: PriceMap,
}

impl Seed for Oracle {
    const SEED: &'static [u8] = b"oracle";
}

impl Oracle {
    /// Initialize the [`Oracle`].
    pub(crate) fn init(&mut self, bump: u8, index: u8) {
        self.primary.clear();
        self.bump = bump;
        self.index = index;
    }

    /// Set prices from remaining accounts.
    pub(crate) fn set_prices_from_remaining_accounts<'info>(
        &mut self,
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
        let clock = Clock::get()?;
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
            let price = match &program {
                PriceProviderProgram::Chainlink(program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    Chainlink::check_and_get_chainlink_price(&clock, program, token_config, feed)?
                }
                PriceProviderProgram::Pyth(_program, kind) => {
                    let feed_id = token_config.get_feed(kind)?;
                    // We have to check the `feed_id` because the `feed` account is provided by user,
                    // and one can provide a feed account for other assets that still be valid.
                    Pyth::check_and_get_price(&clock, token_config, feed, Some(&feed_id))?
                }
                PriceProviderProgram::PythLegacy(_program, kind) => {
                    require_eq!(
                        token_config.get_feed(kind)?,
                        feed.key(),
                        DataStoreError::InvalidPriceFeedAccount
                    );
                    // We don't have to check the `feed_id` because the `feed` account is set by keeper.
                    Pyth::check_and_get_price(&clock, token_config, feed, None)?
                }
            };
            self.primary.set(token, price)?;
        }
        Ok(())
    }

    /// Clear all prices.
    pub(crate) fn clear_all_prices(&mut self) {
        self.primary.clear();
    }

    /// Run a function inside the scope with oracle prices set.
    pub fn with_oracle_prices<'info, T>(
        &mut self,
        provider: &'info Interface<'info, PriceProvider>,
        map: &TokenConfigMap,
        tokens: &[Pubkey],
        remaining_accounts: &'info [AccountInfo<'info>],
        f: impl FnOnce(&Self, &'info [AccountInfo<'info>]) -> Result<T>,
    ) -> Result<T> {
        require_gte!(
            remaining_accounts.len(),
            tokens.len(),
            ErrorCode::AccountNotEnoughKeys
        );
        let feeds = &remaining_accounts[..tokens.len()];
        let remaining_accounts = &remaining_accounts[tokens.len()..];
        self.set_prices_from_remaining_accounts(provider, map, tokens, feeds)?;
        let output = f(self, remaining_accounts)?;
        self.clear_all_prices();
        Ok(output)
    }
}

/// The Chainlink Program.
pub struct Chainlink;

impl Id for Chainlink {
    fn id() -> Pubkey {
        chainlink_solana::ID
    }
}

impl Chainlink {
    /// Check and get latest chainlink price from data feed.
    pub(crate) fn check_and_get_chainlink_price<'info>(
        clock: &Clock,
        chainlink_program: &AccountInfo<'info>,
        token_config: &TokenConfig,
        feed: &AccountInfo<'info>,
    ) -> Result<Price> {
        let round = chainlink_solana::latest_round_data(chainlink_program.clone(), feed.clone())?;
        let decimals =
            chainlink_solana::decimals(chainlink_program.to_account_info(), feed.clone())?;
        Self::check_and_get_price_from_round(clock, &round, decimals, token_config)
    }

    /// Check and get price from the round data.
    fn check_and_get_price_from_round(
        clock: &Clock,
        round: &chainlink_solana::Round,
        decimals: u8,
        token_config: &TokenConfig,
    ) -> Result<Price> {
        let chainlink_solana::Round {
            answer, timestamp, ..
        } = round;
        require_gt!(*answer, 0, DataStoreError::InvalidPriceFeedPrice);
        let timestamp = *timestamp as i64;
        let current = clock.unix_timestamp;
        if current > timestamp && current - timestamp > token_config.heartbeat_duration.into() {
            return Err(DataStoreError::PriceFeedNotUpdated.into());
        }
        let price = Decimal::try_from_price(
            *answer as u128,
            decimals,
            token_config.token_decimals,
            token_config.precision,
        )
        .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?;
        Ok(Price {
            min: price,
            max: price,
        })
    }
}

/// The Pyth receiver program.
pub struct Pyth;

impl Id for Pyth {
    fn id() -> Pubkey {
        pyth_solana_receiver_sdk::ID
    }
}

impl Pyth {
    fn check_and_get_price<'info>(
        clock: &Clock,
        token_config: &TokenConfig,
        feed: &'info AccountInfo<'info>,
        feed_id: Option<&Pubkey>,
    ) -> Result<Price> {
        #[derive(AnchorSerialize, AnchorDeserialize, Clone)]
        struct PythPriceUpdate(PriceUpdateV2);

        static PYTH_IDS: [Pubkey; 2] = [pyth_solana_receiver_sdk::ID, PYTH_LEGACY_ID];

        impl Owners for PythPriceUpdate {
            fn owners() -> &'static [Pubkey] {
                &PYTH_IDS
            }
        }

        impl AccountSerialize for PythPriceUpdate {}

        impl AccountDeserialize for PythPriceUpdate {
            fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
                Ok(Self(PriceUpdateV2::try_deserialize_unchecked(buf)?))
            }

            fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
                Ok(Self(PriceUpdateV2::try_deserialize(buf)?))
            }
        }

        let feed = InterfaceAccount::<PythPriceUpdate>::try_from(feed)?;
        let feed_id = feed_id
            .map(|id| id.to_bytes())
            .unwrap_or_else(|| feed.0.price_message.feed_id);
        let price = feed.0.get_price_no_older_than(
            clock,
            token_config.heartbeat_duration.into(),
            &feed_id,
        )?;
        let mid_price: u64 = price
            .price
            .try_into()
            .map_err(|_| DataStoreError::NegativePrice)?;
        // FIXME: use min and max price when ready.
        let _min_price = mid_price
            .checked_sub(price.conf)
            .ok_or(DataStoreError::NegativePrice)?;
        let _max_price = mid_price
            .checked_add(price.conf)
            .ok_or(DataStoreError::PriceOverflow)?;
        Ok(Price {
            min: Self::price_value_to_decimal(mid_price, price.exponent, token_config)?,
            max: Self::price_value_to_decimal(mid_price, price.exponent, token_config)?,
        })
    }

    fn price_value_to_decimal(
        mut value: u64,
        exponent: i32,
        token_config: &TokenConfig,
    ) -> Result<Decimal> {
        // actual price == value * 10^exponent
        // - If `exponent` is not positive, then the `decimals` is set to `-exponent`.
        // - Otherwise, we should use `value * 10^exponent` as `price` argument, and let `decimals` be `0`.
        let decimals: u8 = if exponent <= 0 {
            (-exponent)
                .try_into()
                .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?
        } else {
            let factor = 10u64
                .checked_pow(exponent as u32)
                .ok_or(DataStoreError::InvalidPriceFeedPrice)?;
            value = value
                .checked_mul(factor)
                .ok_or(DataStoreError::PriceOverflow)?;
            0
        };
        let price = Decimal::try_from_price(
            value as u128,
            decimals,
            token_config.token_decimals,
            token_config.precision,
        )
        .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?;
        Ok(price)
    }
}

/// The legacy Pyth program.
pub struct PythLegacy;

/// The address of legacy Pyth program.
#[cfg(not(feature = "devnet"))]
pub const PYTH_LEGACY_ID: Pubkey = Pubkey::new_from_array([
    220, 229, 235, 225, 228, 156, 59, 159, 17, 76, 181, 84, 76, 80, 169, 158, 192, 214, 146, 214,
    63, 86, 121, 90, 224, 41, 172, 131, 217, 234, 139, 226,
]);

#[cfg(feature = "devnet")]
pub const PYTH_LEGACY_ID: Pubkey = Pubkey::new_from_array([
    10, 26, 152, 51, 163, 118, 85, 43, 86, 183, 202, 13, 237, 25, 41, 23, 0, 87, 232, 39, 160, 198,
    39, 244, 182, 71, 185, 238, 144, 153, 175, 180,
]);

impl Id for PythLegacy {
    fn id() -> Pubkey {
        PYTH_LEGACY_ID
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
#[derive(Clone, Copy, Default, TryFromPrimitive, PartialEq, Eq)]
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
    pub(crate) fn parse_feed_account(&self, feed: &Pubkey) -> Pubkey {
        use crate::utils::pyth::find_pyth_feed_account;

        match self {
            Self::Pyth => find_pyth_feed_account(0, feed.to_bytes()).0,
            Self::Chainlink | Self::PythLegacy => *feed,
        }
    }

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
