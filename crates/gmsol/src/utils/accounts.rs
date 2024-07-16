use std::ops::Deref;

use anchor_client::{
    anchor_lang::{AccountDeserialize, Discriminator},
    solana_client::{
        client_error::ClientError,
        nonblocking::rpc_client::RpcClient,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, RpcFilterType},
        rpc_request::{RpcError, RpcRequest},
        rpc_response::{Response, RpcApiVersion, RpcKeyedAccount, RpcResponseContext},
    },
    solana_sdk::{
        account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer,
    },
    Program,
};
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;

/// Program Accounts Config.
#[derive(Debug, Default)]
pub struct ProgramAccountsConfigForRpc {
    /// Filters.
    pub filters: Option<Vec<RpcFilterType>>,
    /// Account Config.
    pub account_config: RpcAccountInfoConfig,
}

/// With Context.
#[derive(Debug)]
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

/// Get program accounts with context.
///
/// ## Note
/// This function only supports RPC Node versions `>= 1.17`.
pub async fn get_program_accounts_with_context(
    client: &RpcClient,
    program: &Pubkey,
    mut config: ProgramAccountsConfigForRpc,
) -> crate::Result<WithContext<Vec<(Pubkey, Account)>>> {
    let commitment = config
        .account_config
        .commitment
        .unwrap_or_else(|| client.commitment());
    config.account_config.commitment = Some(commitment);
    let config = RpcProgramAccountsConfig {
        filters: config.filters,
        account_config: config.account_config,
        with_context: Some(true),
    };

    let res = client
        .send::<Response<Vec<RpcKeyedAccount>>>(
            RpcRequest::GetProgramAccounts,
            json!([program.to_string(), config]),
        )
        .await
        .map_err(anchor_client::ClientError::from)?;
    let accounts = parse_keyed_accounts(res.value, RpcRequest::GetProgramAccounts)?;
    let context = res.context;
    Ok(WithContext {
        context,
        value: accounts,
    })
}

/// Program Accounts Config.
#[derive(Debug, Default)]
pub struct ProgramAccountsConfig {
    /// Whether to skip the account type filter.
    pub skip_account_type_filter: bool,
    /// Commitment.
    pub commitment: Option<CommitmentConfig>,
    /// Min context slot.
    pub min_context_slot: Option<u64>,
}

/// Returns all program accounts of the given type matching the specified filters as
/// an iterator, along with context. Deserialization is executed lazily.
pub async fn accounts_lazy_with_context<
    T: AccountDeserialize + Discriminator,
    C: Deref<Target = impl Signer> + Clone,
>(
    program: &Program<C>,
    filters: impl IntoIterator<Item = RpcFilterType>,
    config: ProgramAccountsConfig,
) -> crate::Result<WithContext<impl Iterator<Item = crate::Result<(Pubkey, T)>>>> {
    let ProgramAccountsConfig {
        skip_account_type_filter,
        commitment,
        min_context_slot,
    } = config;
    let filters = (!skip_account_type_filter)
        .then(|| RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &T::discriminator())))
        .into_iter()
        .chain(filters)
        .collect::<Vec<_>>();
    let config = ProgramAccountsConfigForRpc {
        filters: (!filters.is_empty()).then_some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment,
            min_context_slot,
            ..Default::default()
        },
    };
    let client = program.async_rpc();
    let res = get_program_accounts_with_context(&client, &program.id(), config).await?;
    Ok(res.map(|accounts| {
        accounts
            .into_iter()
            .map(|(key, account)| Ok((key, T::try_deserialize(&mut (&account.data as &[u8]))?)))
    }))
}

fn parse_keyed_accounts(
    accounts: Vec<RpcKeyedAccount>,
    request: RpcRequest,
) -> crate::Result<Vec<(Pubkey, Account)>> {
    let mut pubkey_accounts: Vec<(Pubkey, Account)> = Vec::with_capacity(accounts.len());
    for RpcKeyedAccount { pubkey, account } in accounts.into_iter() {
        let pubkey = pubkey
            .parse()
            .map_err(|_| {
                ClientError::new_with_request(
                    RpcError::ParseError("Pubkey".to_string()).into(),
                    request,
                )
            })
            .map_err(anchor_client::ClientError::from)?;
        pubkey_accounts.push((
            pubkey,
            account
                .decode()
                .ok_or_else(|| {
                    ClientError::new_with_request(
                        RpcError::ParseError("Account from rpc".to_string()).into(),
                        request,
                    )
                })
                .map_err(anchor_client::ClientError::from)?,
        ));
    }
    Ok(pubkey_accounts)
}
