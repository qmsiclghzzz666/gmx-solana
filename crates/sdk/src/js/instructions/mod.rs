use serde::{Deserialize, Serialize};
use solana_sdk::transaction::VersionedTransaction;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

/// Create order.
pub mod create_order;

/// A JS binding for transactions.
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
