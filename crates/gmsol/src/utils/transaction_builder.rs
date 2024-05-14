use std::ops::Deref;

use anchor_client::solana_sdk::{packet::PACKET_DATA_SIZE, signature::Signature, signer::Signer};

use super::{transaction_size, RpcBuilder};

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
    /// Push a [`RpcBuilder`] with options.
    pub fn try_push_with_opts(
        &mut self,
        mut rpc: RpcBuilder<'a, C>,
        new_transaction: bool,
    ) -> Result<&mut Self, (RpcBuilder<'a, C>, crate::Error)> {
        if self.builders.is_empty() || new_transaction {
            tracing::debug!("adding to a new tx");
            self.builders.push(rpc);
        } else {
            let last = self.builders.last_mut().unwrap();
            let mut ixs_after_merge = last.instructions(false);
            ixs_after_merge.append(&mut rpc.instructions(true));
            let size_after_merge = transaction_size(&ixs_after_merge, true);
            if size_after_merge <= PACKET_DATA_SIZE {
                tracing::debug!(size_after_merge, "adding to the last tx");
                last.try_merge(&mut rpc).map_err(|err| (rpc, err))?;
            } else {
                tracing::debug!(
                    size_after_merge,
                    "exceed packet data size limit, adding to a new tx"
                );
                self.builders.push(rpc);
            }
        }
        Ok(self)
    }

    /// Push a [`RpcBuilder`].
    #[inline]
    pub fn try_push(
        &mut self,
        rpc: RpcBuilder<'a, C>,
    ) -> Result<&mut Self, (RpcBuilder<'a, C>, crate::Error)> {
        self.try_push_with_opts(rpc, false)
    }

    /// Push [`RpcBuilder`]s.
    pub fn try_push_many(
        &mut self,
        rpcs: impl IntoIterator<Item = RpcBuilder<'a, C>>,
    ) -> crate::Result<&mut Self> {
        for rpc in rpcs {
            self.try_push(rpc)?;
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
        for (idx, builder) in self.builders.into_iter().enumerate() {
            tracing::debug!(
                size = builder.transaction_size(false),
                "sending transaction {idx}"
            );
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
