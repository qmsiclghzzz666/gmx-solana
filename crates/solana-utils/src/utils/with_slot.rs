/// With Slot.
#[derive(Debug, Clone, Copy)]
pub struct WithSlot<T> {
    /// Slot.
    slot: u64,
    /// Value.
    value: T,
}

impl<T> WithSlot<T> {
    /// Create a new [`WithSlot`].
    pub fn new(slot: u64, value: T) -> Self {
        Self { slot, value }
    }

    /// Get slot.
    pub fn slot(&self) -> u64 {
        self.slot
    }

    /// Get value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Into value.
    pub fn into_value(self) -> T {
        self.split().1
    }

    /// Apply a function on the value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithSlot<U> {
        WithSlot {
            slot: self.slot,
            value: (f)(self.value),
        }
    }

    /// Split.
    pub fn split(self) -> (u64, T) {
        (self.slot, self.value)
    }
}

impl<T, E> WithSlot<Result<T, E>> {
    /// Transpose.
    pub fn transpose(self) -> Result<WithSlot<T>, E> {
        match self.value {
            Ok(value) => Ok(WithSlot {
                slot: self.slot,
                value,
            }),
            Err(err) => Err(err),
        }
    }
}
