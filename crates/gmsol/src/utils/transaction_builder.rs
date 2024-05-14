use std::ops::Deref;

use anchor_client::solana_sdk::{signature::Signature, signer::Signer};

use super::RpcBuilder;

/// Build transactions from [`RpcBuilder`].
pub struct TransactionBuilder<'a, C> {
    builders: Vec<RpcBuilder<'a, C>>,
}

impl<'a, C> Default for TransactionBuilder<'a, C> {
    fn default() -> Self {
        Self {
            builders: Default::default(),
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> TransactionBuilder<'a, C> {
    /// Push a [`RpcBuilder`].
    pub fn try_push(
        &mut self,
        mut rpc: RpcBuilder<'a, C>,
        new_transaction: bool,
    ) -> Result<&mut Self, (RpcBuilder<'a, C>, crate::Error)> {
        if self.builders.is_empty() || new_transaction {
            self.builders.push(rpc);
        } else {
            // TODO: calculate if reaches the transaction size limit.
            self.builders
                .last_mut()
                .unwrap()
                .try_merge(&mut rpc)
                .map_err(|err| (rpc, err))?;
        }
        Ok(self)
    }

    /// Get back all collected [`RpcBuilder`]s.
    pub fn into_builders(self) -> Vec<RpcBuilder<'a, C>> {
        self.builders
    }

    /// Send all in order and returns the signatures of the success transactions.
    pub async fn send_all(self) -> Result<Vec<Signature>, (Vec<Signature>, crate::Error)> {
        let mut signatures = Vec::with_capacity(self.builders.len());
        let mut error = None;
        for builder in self.builders {
            match builder.build().send().await {
                Ok(signature) => {
                    signatures.push(signature);
                }
                Err(err) => {
                    error = Some(err.into());
                    break;
                }
            }
        }
        match error {
            None => Ok(signatures),
            Some(err) => Err((signatures, err)),
        }
    }
}

impl<T> From<(T, crate::Error)> for crate::Error {
    fn from(value: (T, crate::Error)) -> Self {
        value.1
    }
}
