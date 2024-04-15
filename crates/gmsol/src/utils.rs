use anchor_client::{
    anchor_lang::prelude::borsh::BorshDeserialize,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction},
    solana_sdk::pubkey::Pubkey,
};

use base64::{prelude::BASE64_STANDARD, Engine};

/// View the return data by simulating the transaction.
pub async fn view<T: BorshDeserialize>(
    client: &RpcClient,
    transaction: &impl SerializableTransaction,
) -> crate::Result<T> {
    let res = client
        .simulate_transaction(transaction)
        .await
        .map_err(anchor_client::ClientError::from)?;
    let (data, _encoding) = res
        .value
        .return_data
        .ok_or(crate::Error::MissingReturnData)?
        .data;
    let decoded = BASE64_STANDARD.decode(data)?;
    let output = T::deserialize_reader(&mut decoded.as_slice())?;
    Ok(output)
}

/// A workaround to deserialize "zero-copy" account data.
///
/// See [anchort#2689](https://github.com/coral-xyz/anchor/issues/2689) for more information.
pub async fn try_deserailize_account<T>(client: &RpcClient, pubkey: &Pubkey) -> crate::Result<T>
where
    T: anchor_client::anchor_lang::ZeroCopy,
{
    use anchor_client::{
        anchor_lang::error::{Error, ErrorCode},
        ClientError,
    };

    let data = client
        .get_account_data(pubkey)
        .await
        .map_err(anchor_client::ClientError::from)?;
    let disc = T::discriminator();
    if data.len() < disc.len() {
        return Err(ClientError::from(Error::from(ErrorCode::AccountDiscriminatorNotFound)).into());
    }
    let given_disc = &data[..8];
    if disc != given_disc {
        return Err(ClientError::from(Error::from(ErrorCode::AccountDiscriminatorMismatch)).into());
    }
    let end = std::mem::size_of::<T>() + 8;
    if data.len() < end {
        return Err(ClientError::from(Error::from(ErrorCode::AccountDidNotDeserialize)).into());
    }
    let data_without_discriminator = data[8..end].to_vec();
    Ok(*bytemuck::try_from_bytes(&data_without_discriminator).map_err(crate::Error::Bytemuck)?)
}
