use time::OffsetDateTime;

fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

pub(super) struct AsClock<'a> {
    last: &'a i64,
}

impl AsClock<'_> {
    /// Passed in seconds.
    pub(super) fn passed_in_seconds(&mut self) -> gmsol_model::Result<u64> {
        let current = now();
        let duration = current.saturating_sub(*self.last);
        if duration > 0 {
            Ok(duration as u64)
        } else {
            Ok(0)
        }
    }
}

impl<'a> From<&'a i64> for AsClock<'a> {
    fn from(last: &'a i64) -> Self {
        Self { last }
    }
}

/// Clock-related operations.
pub(super) struct AsClockMut<'a> {
    last: &'a mut i64,
}

impl AsClockMut<'_> {
    /// Just passed in seconds.
    pub(super) fn just_passed_in_seconds(&mut self) -> gmsol_model::Result<u64> {
        let current = now();
        let duration = current.saturating_sub(*self.last);
        if duration > 0 {
            *self.last = current;
            Ok(duration as u64)
        } else {
            Ok(0)
        }
    }
}

impl<'a> From<&'a mut i64> for AsClockMut<'a> {
    fn from(last: &'a mut i64) -> Self {
        Self { last }
    }
}
