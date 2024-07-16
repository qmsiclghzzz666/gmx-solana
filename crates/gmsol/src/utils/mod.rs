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

/// Workaround for optional accounts.
pub mod optional;

/// Zero-copy account workaround.
pub mod zero_copy;

/// Program accounts.
pub mod accounts;

pub use self::{
    compute_budget::ComputeBudget,
    instruction::serialize_instruction,
    optional::fix_optional_account_metas,
    rpc_builder::RpcBuilder,
    signer::{shared_signer, SignerRef},
    transaction_builder::TransactionBuilder,
    transaction_size::transaction_size,
    zero_copy::try_deserailize_zero_copy_account,
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
    if let Some(error) = res.value.err {
        return Err(crate::Error::unknown(format!(
            "error={error}, logs={:#?}",
            res.value.logs,
        )));
    }
    let (data, _encoding) = res
        .value
        .return_data
        .ok_or(crate::Error::MissingReturnData)?
        .data;
    let decoded = BASE64_STANDARD.decode(data)?;
    let output = T::deserialize_reader(&mut decoded.as_slice())?;
    Ok(output)
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

pub use gmsol_store::constants::EVENT_AUTHORITY_SEED;
