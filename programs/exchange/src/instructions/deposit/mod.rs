mod create;
mod execute;

pub use create::*;
pub use execute::*;

/// Max Execution Fee (lamports).
// TODO: make it configurable.
pub const MAX_EXECUTION_FEE: u64 = 5001;
