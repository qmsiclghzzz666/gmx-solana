use gmsol_programs::gmsol_treasury::{
    accounts::{Config, GtBank, TreasuryVaultConfig},
    types::{TokenBalance, TokenConfig},
};
use gmsol_utils::{gt::GtBankFlags, token_config::TokenFlag, token_config::TokenMapAccess};
use indexmap::IndexMap;
use solana_sdk::pubkey::Pubkey;

use crate::utils::{Amount, Value};

use super::StringPubkey;

/// Serializable version of treausry config type [`Config`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeTreasury {
    /// Associated store address.
    pub store: StringPubkey,
    /// Authorized treasury vault config address.
    pub vault_config: StringPubkey,
    /// The proportion of the received fee allocated to GT Bank.
    pub gt_factor: Value,
    /// Maximum daily share of Treasury value for GT buyback.
    pub buyback_factor: Value,
}

impl<'a> From<&'a Config> for SerdeTreasury {
    fn from(config: &'a Config) -> Self {
        Self {
            store: config.store.into(),
            vault_config: config.treasury_vault_config.into(),
            gt_factor: Value::from_u128(config.gt_factor),
            buyback_factor: Value::from_u128(config.buyback_factor),
        }
    }
}

/// Serializable version of [`TreasuryVaultConfig`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeTreasuryVaultConfig {
    /// The index of the vault.
    pub index: u16,
    /// The associated treasury config address.
    pub config: StringPubkey,
    /// The config for each token.
    pub tokens: IndexMap<StringPubkey, SerdeTreausryTokenConfig>,
}

impl<'a> From<&'a TreasuryVaultConfig> for SerdeTreasuryVaultConfig {
    fn from(config: &'a TreasuryVaultConfig) -> Self {
        Self {
            index: config.index,
            config: config.config.into(),
            tokens: config
                .tokens
                .entries()
                .map(|(k, v)| (Pubkey::new_from_array(*k).into(), v.into()))
                .collect(),
        }
    }
}

/// Serializable version of [`TokenConfig`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeTreausryTokenConfig {
    /// Whether the deposit of the token is allowed.
    pub is_deposit_allowed: bool,
    /// Whether the withdrawal of the token is allowed.
    pub is_withdrawal_allowed: bool,
}

impl<'a> From<&'a TokenConfig> for SerdeTreausryTokenConfig {
    fn from(config: &'a TokenConfig) -> Self {
        Self {
            is_deposit_allowed: config.flags.get_flag(TokenFlag::AllowDeposit),
            is_withdrawal_allowed: config.flags.get_flag(TokenFlag::AllowWithdrawal),
        }
    }
}

/// Serializable version of [`GtBank`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeGtBank {
    /// Whether the GT bank is initialized.
    pub is_initialized: bool,
    /// Whether the GT bank is confirmed.
    pub is_confirmed: bool,
    /// Whether the GT bank is synced after confirmation.
    pub is_synced_after_confirmation: bool,
    /// The associated treasury vault config address.
    pub treasury_vault_config: StringPubkey,
    /// The associated GT exchange vault address.
    pub gt_exchange_vault: StringPubkey,
    /// The remaining confirmed GT amount.
    pub remaining_confirmed_gt_amount: Amount,
    /// Token balances.
    pub balances: IndexMap<StringPubkey, SerdeGtBankBalance>,
}

impl SerdeGtBank {
    /// Create from [`GtBank`].
    pub fn from_gt_bank(
        bank: &GtBank,
        gt_decimals: u8,
        token_map: &impl TokenMapAccess,
    ) -> crate::Result<Self> {
        Ok(Self {
            is_initialized: bank.flags.get_flag(GtBankFlags::Initialized),
            is_confirmed: bank.flags.get_flag(GtBankFlags::Confirmed),
            is_synced_after_confirmation: bank.flags.get_flag(GtBankFlags::SyncedAfterConfirmation),
            treasury_vault_config: bank.treasury_vault_config.into(),
            gt_exchange_vault: bank.gt_exchange_vault.into(),
            remaining_confirmed_gt_amount: Amount::from_u64(
                bank.remaining_confirmed_gt_amount,
                gt_decimals,
            ),
            balances: bank
                .balances
                .entries()
                .map(|(k, v)| {
                    let token = Pubkey::new_from_array(*k);
                    let decimals = token_map
                        .get(&token)
                        .ok_or_else(|| crate::Error::NotFound)?
                        .token_decimals;
                    Ok((token.into(), SerdeGtBankBalance::from_balance(v, decimals)))
                })
                .collect::<crate::Result<_>>()?,
        })
    }
}

/// Serializable version of [`GtBank`].
#[derive(Debug, Clone)]
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
pub struct SerdeGtBankBalance {
    /// Token amount.
    pub amount: Amount,
}

impl SerdeGtBankBalance {
    /// Create from [`TokenBalance`].
    pub fn from_balance(balance: &TokenBalance, token_decimals: u8) -> Self {
        Self {
            amount: Amount::from_u64(balance.amount, token_decimals),
        }
    }
}
