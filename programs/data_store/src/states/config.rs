use anchor_lang::prelude::*;

use super::{common::MapStore, Amount, Factor, Seed};

/// Config.
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Config {
    /// Bump.
    pub bump: u8,
    /// Factors.
    factors: MapStore<[u8; 32], u128, 32>,
    /// Amounts or seconds.
    amounts: MapStore<[u8; 32], u64, 32>,
}

impl Seed for Config {
    const SEED: &'static [u8] = b"config";
}

impl Config {
    /// Insert a new factor.
    pub fn insert_factor(&mut self, namespace: &str, key: &str, factor: u128) -> Option<Factor> {
        self.factors.insert(namespace, key, factor)
    }

    /// Insert a new amount.
    pub fn insert_amount(&mut self, namespace: &str, key: &str, amount: u64) -> Option<Amount> {
        self.amounts.insert(namespace, key, amount)
    }
}
