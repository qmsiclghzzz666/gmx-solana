use anchor_client::solana_client::rpc_response::{Response, RpcApiVersion, RpcResponseContext};

/// With Context.
#[derive(Debug, Clone)]
pub struct WithContext<T> {
    /// Context.
    context: RpcResponseContext,
    /// Value.
    value: T,
}

impl<T> WithContext<T> {
    /// Apply a function on the value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithContext<U> {
        WithContext {
            context: self.context,
            value: (f)(self.value),
        }
    }

    /// Into value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Get a refercne to the value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Get response slot.
    pub fn slot(&self) -> u64 {
        self.context.slot
    }

    /// Get API version.
    pub fn api_version(&self) -> Option<&RpcApiVersion> {
        self.context.api_version.as_ref()
    }
}

impl<T, E> WithContext<Result<T, E>> {
    /// Transpose.
    pub fn transpose(self) -> Result<WithContext<T>, E> {
        match self.value {
            Ok(value) => Ok(WithContext {
                context: self.context,
                value,
            }),
            Err(err) => Err(err),
        }
    }
}

impl<T> From<Response<T>> for WithContext<T> {
    fn from(res: Response<T>) -> Self {
        Self {
            context: res.context,
            value: res.value,
        }
    }
}

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
        self.value
    }

    /// Apply a function on the value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithSlot<U> {
        WithSlot {
            slot: self.slot,
            value: (f)(self.value),
        }
    }
}

impl<T> From<WithContext<T>> for WithSlot<T> {
    fn from(value: WithContext<T>) -> Self {
        Self {
            slot: value.context.slot,
            value: value.value,
        }
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
