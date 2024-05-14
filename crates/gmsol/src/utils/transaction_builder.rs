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

    /// Send all.
    pub async fn send_all(self) -> crate::Result<Vec<Signature>> {
        let mut signatures = Vec::with_capacity(self.builders.len());
        for builder in self.builders {
            let signature = builder.build().send().await?;
            signatures.push(signature);
        }
        Ok(signatures)
    }
}

impl<'a, C> From<(RpcBuilder<'a, C>, crate::Error)> for crate::Error {
    fn from(value: (RpcBuilder<'a, C>, crate::Error)) -> Self {
        value.1
    }
}
