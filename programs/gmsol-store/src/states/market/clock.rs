use anchor_lang::{
    error::Error,
    solana_program::{clock::Clock, sysvar::Sysvar},
};

/// Clock-related operations.
pub struct AsClock<'a> {
    last: &'a mut i64,
}

impl<'a> AsClock<'a> {
    /// Just passed in seconds.
    pub fn just_passed_in_seconds(&mut self) -> gmx_core::Result<u64> {
        let current = Clock::get().map_err(Error::from)?.unix_timestamp;
        let duration = current.saturating_sub(*self.last);
        if duration > 0 {
            *self.last = current;
        }
        Ok(duration as u64)
    }
}

impl<'a> From<&'a mut i64> for AsClock<'a> {
    fn from(last: &'a mut i64) -> Self {
        Self { last }
    }
}
