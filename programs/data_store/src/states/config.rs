use anchor_lang::prelude::*;

use super::{common::map::MapStore, Amount, Factor, Seed};

/// Factor Keys.
#[derive(num_enum::TryFromPrimitive, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(u8)]
pub enum FactorKey {
    /// Ref Price Deviation Factor.
    RefPriceDeviation,
}

/// Amount Keys.
#[derive(num_enum::TryFromPrimitive, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(u8)]
pub enum AmountKey {
    /// Max Age.
    MaxAge,
    /// Request Expiration Time.
    RequestExpirationTime,
    /// Max Oracle Timestamp Range.
    MaxOracleTimestampRange,
}

/// Config.
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Config {
    /// Bump.
    pub bump: u8,
    /// Factors.
    factors: MapStore<u8, u128, 32>,
    /// Amounts or seconds.
    amounts: MapStore<u8, u64, 32>,
}

impl Seed for Config {
    const SEED: &'static [u8] = b"config";
}

impl Config {
    /// Insert a new factor.
    pub fn insert_factor(&mut self, key: FactorKey, factor: u128) -> Option<Factor> {
        self.factors
            .as_map_mut()
            .insert(key as u8, factor)
            .map(|(_, v)| v)
    }

    /// Insert a new amount.
    pub fn insert_amount(&mut self, key: AmountKey, amount: u64) -> Option<Amount> {
        self.amounts
            .as_map_mut()
            .insert(key as u8, amount)
            .map(|(_, v)| v)
    }
}
