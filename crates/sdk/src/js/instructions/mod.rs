use serde::{Deserialize, Serialize};
use solana_sdk::transaction::VersionedTransaction;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

/// Create order.
pub mod create_order;

/// A JS binding for transactions.
#[wasm_bindgen]
pub struct Transactions(Vec<Vec<VersionedTransaction>>);

/// Serialized transactions.
#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct SerializedTransactions(Vec<Vec<Vec<u8>>>);

#[wasm_bindgen]
impl Transactions {
    /// Serialize to serialized transaciton list.
    pub fn serialize(&self) -> crate::Result<SerializedTransactions> {
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
        Ok(SerializedTransactions(serialized))
    }
}
