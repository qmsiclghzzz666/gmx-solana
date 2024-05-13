use std::ops::Deref;

use anchor_client::{
    anchor_lang::{prelude::borsh::BorshDeserialize, InstructionData, ToAccountMetas},
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signer::Signer,
    },
    Program,
};

use anchor_spl::associated_token::get_associated_token_address;
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

/// A wrapper of [`RequestBuilder`](anchor_client::RequestBuilder)
/// better instruction insertion methods.
#[must_use]
pub struct RpcBuilder<'a, C> {
    program_id: Pubkey,
    builder_with_pre_instructions: anchor_client::RequestBuilder<'a, C>,
    accounts: Vec<AccountMeta>,
    instruction_data: Option<Vec<u8>>,
    post_instructions: Vec<Instruction>,
    compute_budget: Option<ComputeBudget>,
}

/// Compute Budget.
#[derive(Debug, Clone, Copy)]
pub struct ComputeBudget {
    limit_units: u32,
    price_micro_lamports: u64,
}

impl Default for ComputeBudget {
    fn default() -> Self {
        Self {
            limit_units: 200_000,
            price_micro_lamports: 100_000,
        }
    }
}

impl ComputeBudget {
    /// Set compute units limit.
    pub fn with_limit(mut self, units: u32) -> Self {
        self.limit_units = units;
        self
    }

    /// Set compute unit price.
    pub fn with_price(mut self, micro_lamports: u64) -> Self {
        self.price_micro_lamports = micro_lamports;
        self
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RpcBuilder<'a, C> {
    /// Create a new [`RpcBuilder`] from [`Program`].
    pub fn new(program: &'a Program<C>) -> Self {
        Self {
            program_id: program.id(),
            builder_with_pre_instructions: program.request(),
            accounts: Default::default(),
            instruction_data: None,
            post_instructions: Default::default(),
            compute_budget: None,
        }
    }

    /// Set payer.
    pub fn payer(mut self, payer: C) -> Self {
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.payer(payer);
        self
    }

    /// Set cluster.
    pub fn cluster(mut self, url: &str) -> Self {
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.cluster(url);
        self
    }

    /// Set commiment options.
    pub fn options(mut self, options: CommitmentConfig) -> Self {
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.options(options);
        self
    }

    /// Add a signer to the signer list.
    pub fn signer(mut self, signer: &'a dyn Signer) -> Self {
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.signer(signer);
        self
    }

    /// Set program id.
    pub fn program(mut self, program_id: Pubkey) -> Self {
        self.program_id = program_id;
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.program(program_id);
        self
    }

    /// Set accounts for the rpc method.
    pub fn accounts(mut self, accounts: impl ToAccountMetas) -> Self {
        let mut metas = accounts.to_account_metas(None);
        self.accounts.append(&mut metas);
        self
    }

    /// Set arguments for the rpc method.
    pub fn args(mut self, args: impl InstructionData) -> Self {
        self.instruction_data = Some(args.data());
        self
    }

    /// Set compute budget.
    pub fn compute_budget(mut self, budget: Option<ComputeBudget>) -> Self {
        self.compute_budget = budget;
        self
    }

    fn get_compute_budget_instructions(&self) -> Vec<Instruction> {
        let Some(budget) = self.compute_budget.as_ref() else {
            return vec![];
        };
        vec![
            ComputeBudgetInstruction::set_compute_unit_limit(budget.limit_units),
            ComputeBudgetInstruction::set_compute_unit_price(budget.price_micro_lamports),
        ]
    }

    /// Get all instructions.
    pub fn instructions(&self) -> Result<Vec<Instruction>, anchor_client::ClientError> {
        let mut instructions = self.get_compute_budget_instructions();
        instructions.append(&mut self.builder_with_pre_instructions.instructions()?);
        if let Some(ix_data) = &self.instruction_data {
            instructions.push(Instruction {
                program_id: self.program_id,
                data: ix_data.clone(),
                accounts: self.accounts.clone(),
            });
        }
        instructions.append(&mut self.post_instructions.clone());
        Ok(instructions)
    }

    /// Build [`RequestBuilder`].
    pub fn build(self) -> Result<anchor_client::RequestBuilder<'a, C>, anchor_client::ClientError> {
        Ok(self
            .instructions()?
            .into_iter()
            .fold(self.builder_with_pre_instructions, |acc, ix| {
                acc.instruction(ix)
            }))
    }

    /// Insert an instruction before the rpc method.
    pub fn pre_instruction(mut self, ix: Instruction) -> Self {
        self.builder_with_pre_instructions = self.builder_with_pre_instructions.instruction(ix);
        self
    }

    /// Insert an instruction after the rpc method.
    pub fn post_isntruction(mut self, ix: Instruction) -> Self {
        self.post_instructions.push(ix);
        self
    }
}
