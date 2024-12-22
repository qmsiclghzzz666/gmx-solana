/// Decoders for Solana datas.
#[cfg(feature = "solana-decoder")]
pub mod solana_decoder;

#[cfg(feature = "solana-decoder")]
pub use solana_decoder::{CPIEventFilter, CPIEvents, TransactionDecoder};
