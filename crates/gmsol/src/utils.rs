use anchor_client::{
    anchor_lang::prelude::borsh::BorshDeserialize,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction},
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
