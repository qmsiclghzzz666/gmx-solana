/// Error type.
pub mod error;

/// Instruction Group.
pub mod instruction_group;

/// Constants.
pub mod constants;

/// Functions for constructing Program Derived Addresses.
pub mod pda;

/// Instruction Builders.
pub mod builders;

/// Utils.
pub mod utils;

/// JavaScript support.
#[cfg(feature = "js")]
pub mod js;

/// Maintains a graph structured with [`MarketModel`](crate::model::MarketModel) as edges.
#[cfg(feature = "market-graph")]
pub mod market_graph;

/// Model support.
pub mod model {
    pub use gmsol_model::*;
    pub use gmsol_programs::model::*;
}

pub use error::Error;
pub use instruction_group::{AtomicInstructionGroup, InstructionGroup, IntoAtomicInstructionGroup};

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

pub use gmsol_programs as programs;
