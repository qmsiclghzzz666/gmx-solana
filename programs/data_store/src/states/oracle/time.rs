use anchor_lang::prelude::*;

use crate::DataStoreError;

use super::Oracle;

/// Validate Oracle Time.
pub trait ValidateOracleTime {
    /// Oracle must be updated after this time.
    fn oracle_updated_after(&self) -> Result<Option<i64>>;

    /// Validate min oracle ts.
    fn validate_min_oracle_ts(&self, oracle: &Oracle) -> Result<()> {
        let Some(after) = self.oracle_updated_after()? else {
            return Ok(());
        };
        require_gte!(
            oracle.min_oracle_ts,
            after,
            DataStoreError::OracleTimestampsAreSmallerThanRequired
        );
        Ok(())
    }

    /// Oracle must be updated before this time.
    fn oracle_updated_before(&self) -> Result<Option<i64>>;

    /// Validate max oracle ts.
    fn validate_max_oracle_ts(&self, oracle: &Oracle) -> Result<()> {
        let Some(before) = self.oracle_updated_before()? else {
            return Ok(());
        };
        require_gte!(
            before,
            oracle.max_oracle_ts,
            DataStoreError::OracleTimestampsAreLargerThanRequired
        );
        Ok(())
    }
}
