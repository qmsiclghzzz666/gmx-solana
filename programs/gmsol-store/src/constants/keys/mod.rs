/// Token config keys.
pub mod token;

/// Global Namespace.
pub const GLOBAL: &str = "global";

// Oracle related config keys.
/// Ref Price Deviation Factor (`factor`).
pub const REF_PRICE_DEVIATION: &str = "ref_price_deviation";
/// Max Age (secs, `amount`).
pub const MAX_AGE: &str = "max_age";
/// Request Expiration Time (secs, `amount`).
pub const REQUEST_EXPIRATION_TIME: &str = "request_expiration_time";
/// Max Oracle Timestamp Range (secs, `amount`).
pub const MAX_ORACLE_TIMESTAMP_RANGE: &str = "max_oracle_timestamp_range";

// Order related global keys.
/// Holding address (`Pubkey`).
pub const HOLDING: &str = "holding";
/// Claimable time window (secs, `amount`)
pub const CLAIMABLE_TIME_WINDOW: &str = "claimable_time_window";
/// Recent time window (secs, `amount`)
pub const RECENT_TIME_WINDOW: &str = "recent_time_window";
