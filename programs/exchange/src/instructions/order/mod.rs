mod create;

/// Max Execution Fee (lamports) for orders.
// TODO: make it configurable.
pub const MAX_ORDER_EXECUTION_FEE: u64 = 5005;

pub use create::*;
