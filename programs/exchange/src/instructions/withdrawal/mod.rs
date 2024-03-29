mod cancel;
mod create;
mod execute;

/// Max Execution Fee (lamports) for withdrawal
// TODO: make it configurable.
pub const MAX_WITHDRAWAL_EXECUTION_FEE: u64 = 5001;
