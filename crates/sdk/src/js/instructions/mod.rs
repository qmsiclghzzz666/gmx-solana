use std::collections::HashMap;

use crate::{
    solana_utils::transaction_group::TransactionGroupOptions as SdkTransactionGroupOptions,
    utils::serde::StringPubkey,
};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::VersionedTransaction;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

/// Create order.
pub mod create_order;

/// A JS version transaction group options.
#[derive(Debug, Serialize, Deserialize, Tsify, Default)]
#[tsify(from_wasm_abi)]
pub struct TransactionGroupOptions {
    #[serde(default)]
    max_transaction_size: Option<usize>,
    #[serde(default)]
    max_instructions_per_tx: Option<usize>,
    #[serde(default)]
    compute_unit_price_micro_lamports: Option<u64>,
    #[serde(default)]
    luts: HashMap<StringPubkey, Vec<StringPubkey>>,
}

impl<'a> From<&'a TransactionGroupOptions> for SdkTransactionGroupOptions {
    fn from(value: &'a TransactionGroupOptions) -> Self {
        let mut options = SdkTransactionGroupOptions::default();
        if let Some(size) = value.max_transaction_size {
            options.max_transaction_size = size;
        }
        if let Some(num) = value.max_instructions_per_tx {
            options.max_instructions_per_tx = num;
        }
        options.compute_unit_price_micro_lamports = value.compute_unit_price_micro_lamports;
        options
    }
}

/// A JS binding for transaction group.
#[wasm_bindgen]
pub struct TransactionGroup(Vec<Vec<VersionedTransaction>>);

/// Serialized transaction group.
#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct SerializedTransactionGroup(Vec<Vec<Vec<u8>>>);

#[wasm_bindgen]
impl TransactionGroup {
    /// Returns serialized transaciton group.
    pub fn serialize(&self) -> crate::Result<SerializedTransactionGroup> {
        let serialized = self
            .0
            .iter()
            .map(|batch| {
                batch
                    .iter()
                    .map(|txn| Ok(bincode::serialize(txn)?))
                    .collect::<crate::Result<Vec<_>>>()
            })
            .collect::<crate::Result<Vec<_>>>()?;
        Ok(SerializedTransactionGroup(serialized))
    }
}
