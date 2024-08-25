use std::sync::Arc;

use anchor_client::{
    anchor_lang::{AccountDeserialize, Discriminator},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::pubkey::Pubkey,
};

/// A workaround to deserialize "zero-copy" account data.
///
/// See [anchort#2689](https://github.com/coral-xyz/anchor/issues/2689) for more information.
pub async fn try_deserailize_zero_copy_account<T>(
    client: &RpcClient,
    pubkey: &Pubkey,
) -> crate::Result<T>
where
    T: anchor_client::anchor_lang::ZeroCopy,
{
    let data = client
        .get_account_data(pubkey)
        .await
        .map_err(anchor_client::ClientError::from)?;

    Ok(gmsol_store::utils::de::try_deserailize(&data)?)
}

/// Workaround for deserializing zero-copy accounts.
pub struct ZeroCopy<T>(pub T);

impl<T> ZeroCopy<T> {
    /// Conver into inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AccountDeserialize for ZeroCopy<T>
where
    T: anchor_client::anchor_lang::ZeroCopy,
{
    fn try_deserialize(buf: &mut &[u8]) -> anchor_client::anchor_lang::Result<Self> {
        let account = gmsol_store::utils::de::try_deserailize(buf)?;
        Ok(Self(account))
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_client::anchor_lang::Result<Self> {
        let account = gmsol_store::utils::de::try_deserailize_unchecked(buf)?;
        Ok(Self(account))
    }
}

impl<T> Discriminator for ZeroCopy<T>
where
    T: Discriminator,
{
    const DISCRIMINATOR: [u8; 8] = T::DISCRIMINATOR;
}

/// Workaround for deserializing zero-copy accounts.
pub struct SharedZeroCopy<T>(pub Arc<T>);

impl<T> SharedZeroCopy<T> {
    /// Conver into inner value.
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> AccountDeserialize for SharedZeroCopy<T>
where
    T: anchor_client::anchor_lang::ZeroCopy,
{
    fn try_deserialize(buf: &mut &[u8]) -> anchor_client::anchor_lang::Result<Self> {
        let account = gmsol_store::utils::de::try_deserailize(buf)?;
        Ok(Self(Arc::new(account)))
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_client::anchor_lang::Result<Self> {
        let account = gmsol_store::utils::de::try_deserailize_unchecked(buf)?;
        Ok(Self(Arc::new(account)))
    }
}

impl<T> Discriminator for SharedZeroCopy<T>
where
    T: Discriminator,
{
    const DISCRIMINATOR: [u8; 8] = T::DISCRIMINATOR;
}
