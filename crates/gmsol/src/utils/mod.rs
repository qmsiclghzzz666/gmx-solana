use std::ops::Deref;

use anchor_client::{
    anchor_lang::prelude::borsh::BorshDeserialize,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};

use anchor_spl::associated_token::get_associated_token_address;
use base64::{prelude::BASE64_STANDARD, Engine};

/// Compute Budget.
pub mod compute_budget;

/// RPC Builder.
pub mod rpc_builder;

/// Transaction Builder.
pub mod transaction_builder;

/// Transaction size.
pub mod transaction_size;

/// Instruction utils.
pub mod instruction;

/// Signer.
pub mod signer;

pub use self::{
    compute_budget::ComputeBudget,
    instruction::serialize_instruction,
    rpc_builder::RpcBuilder,
    signer::{shared_signer, SignerRef},
    transaction_builder::TransactionBuilder,
    transaction_size::transaction_size,
};

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

/// Token Account Params.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokenAccountParams {
    token: Option<Pubkey>,
    token_account: Option<Pubkey>,
}

impl TokenAccountParams {
    /// Set token account.
    pub fn set_token_account(&mut self, account: Pubkey) -> &mut Self {
        self.token_account = Some(account);
        self
    }

    /// Set token.
    pub fn set_token(&mut self, mint: Pubkey) -> &mut Self {
        self.token = Some(mint);
        self
    }

    /// Get token.
    pub fn token(&self) -> Option<&Pubkey> {
        self.token.as_ref()
    }

    /// Get or find associated token account.
    pub fn get_or_find_associated_token_account(&self, owner: Option<&Pubkey>) -> Option<Pubkey> {
        match self.token_account {
            Some(account) => Some(account),
            None => {
                let token = self.token.as_ref()?;
                let owner = owner?;
                Some(get_associated_token_address(owner, token))
            }
        }
    }

    /// Get of fetch token and token account.
    ///
    /// Returns `(token, token_account)` if success.
    pub async fn get_or_fetch_token_and_token_account<S, C>(
        &self,
        program: &Program<C>,
        owner: Option<&Pubkey>,
    ) -> crate::Result<Option<(Pubkey, Pubkey)>>
    where
        C: Deref<Target = S> + Clone,
        S: Signer,
    {
        use anchor_spl::token::TokenAccount;
        match (self.token, self.token_account) {
            (Some(token), Some(account)) => Ok(Some((token, account))),
            (None, Some(account)) => {
                let mint = program.account::<TokenAccount>(account).await?.mint;
                Ok(Some((mint, account)))
            }
            (Some(token), None) => {
                let Some(account) = self.get_or_find_associated_token_account(owner) else {
                    return Err(crate::Error::invalid_argument(
                        "cannot find associated token account: `owner` is not provided",
                    ));
                };
                Ok(Some((token, account)))
            }
            (None, None) => Ok(None),
        }
    }

    /// Returns whether the params is empty.
    pub fn is_empty(&self) -> bool {
        self.token.is_none() && self.token.is_none()
    }
}

/// Event authority SEED.
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";
