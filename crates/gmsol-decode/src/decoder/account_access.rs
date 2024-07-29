use solana_sdk::pubkey::Pubkey;

use crate::decode::visitor::Visitor;

use super::{DecodeError, Decoder};

/// Access an account info.
pub trait AccountAccess {
    /// Get the owner of the account.
    fn owner(&self) -> Result<Pubkey, DecodeError>;

    /// Get the pubkey of the account.
    fn pubkey(&self) -> Result<Pubkey, DecodeError>;

    /// Get the lamports of the account.
    fn lamports(&self) -> Result<u64, DecodeError>;

    /// Get the account data.
    fn data(&self) -> Result<&[u8], DecodeError>;

    /// Get the slot at which the account data was updated.
    fn slot(&self) -> Result<u64, DecodeError>;
}

impl<'a, A: AccountAccess> AccountAccess for &'a A {
    fn owner(&self) -> Result<Pubkey, DecodeError> {
        (**self).owner()
    }

    fn pubkey(&self) -> Result<Pubkey, DecodeError> {
        (**self).pubkey()
    }

    fn lamports(&self) -> Result<u64, DecodeError> {
        (**self).lamports()
    }

    fn data(&self) -> Result<&[u8], DecodeError> {
        (**self).data()
    }

    fn slot(&self) -> Result<u64, DecodeError> {
        (**self).slot()
    }
}

/// Decoder derived from [`AccountAccess`].
#[derive(Debug, Clone)]
pub struct AccountAccessDecoder<A>(A);

impl<A> AccountAccessDecoder<A> {
    /// Create a [`Decoder`] from an [`AccountAccess`].
    pub fn new(account: A) -> Self {
        Self(account)
    }
}

impl<A: AccountAccess> Decoder for AccountAccessDecoder<A> {
    fn decode_account<V>(&self, visitor: V) -> Result<V::Value, DecodeError>
    where
        V: Visitor,
    {
        visitor.visit_account(&self.0)
    }

    fn decode_transaction<V>(&self, _visitor: V) -> Result<V::Value, DecodeError>
    where
        V: Visitor,
    {
        Err(DecodeError::InvalidType(
            "Expecting `Transaction` but found `AccountInfo`".to_string(),
        ))
    }

    fn decode_anchor_cpi_events<V>(&self, _visitor: V) -> Result<V::Value, DecodeError>
    where
        V: Visitor,
    {
        Err(DecodeError::InvalidType(
            "Expecting `AnchorCPIEvents` but found `AccountInfo`".to_string(),
        ))
    }

    fn decode_owned_data<V>(&self, visitor: V) -> Result<V::Value, DecodeError>
    where
        V: Visitor,
    {
        visitor.visit_owned_data(&self.0.owner()?, self.0.data()?)
    }

    fn decode_bytes<V>(&self, visitor: V) -> Result<V::Value, DecodeError>
    where
        V: Visitor,
    {
        visitor.visit_bytes(self.0.data()?)
    }
}
