use std::ops::Deref;

use anchor_client::{
    anchor_lang::{InstructionData, ToAccountMetas},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Signature,
        signer::Signer,
    },
    Program,
};

use super::{compute_budget::ComputeBudget, transaction_size::transaction_size};

/// A wrapper of [`RequestBuilder`](anchor_client::RequestBuilder).
#[must_use]
pub struct RpcBuilder<'a, C, T = ()> {
    output: T,
    program_id: Pubkey,
    payer: Pubkey,
    signers: Vec<&'a dyn Signer>,
    builder: anchor_client::RequestBuilder<'a, C>,
    pre_instructions: Vec<Instruction>,
    accounts: Vec<AccountMeta>,
    instruction_data: Option<Vec<u8>>,
    compute_budget: ComputeBudget,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RpcBuilder<'a, C> {
    /// Create a new [`RpcBuilder`] from [`Program`].
    pub fn new(program: &'a Program<C>) -> Self {
        Self {
            output: (),
            payer: program.payer(),
            signers: Default::default(),
            program_id: program.id(),
            builder: program.request(),
            pre_instructions: Default::default(),
            accounts: Default::default(),
            instruction_data: None,
            compute_budget: ComputeBudget::default(),
        }
    }

    /// Take and construct the RPC instruction if present.
    pub fn take_rpc(&mut self) -> Option<Instruction> {
        let ix_data = self.instruction_data.take()?;
        Some(Instruction {
            program_id: self.program_id,
            data: ix_data,
            accounts: std::mem::take(&mut self.accounts),
        })
    }

    /// Merge other [`RpcBuilder`]. The rpc fields will be empty after merging,
    /// i.e., `take_rpc` will return `None`.
    /// ## Panics
    /// Return if there are any errors.
    /// ## Notes
    /// - All options including `cluster`, `commiment` and `program_id` will still be
    ///   the same of `self` after merging.
    #[inline]
    pub fn merge(mut self, mut other: Self) -> Self {
        self.try_merge(&mut other)
            .unwrap_or_else(|err| panic!("failed to merge: {err}"));
        self
    }

    /// Merge other [`RpcBuilder`]. The rpc fields will be empty after merging,
    /// i.e., `take_rpc` will return `None`.
    /// ## Errors
    /// Return error if the `payer`s are not the same.
    /// ## Notes
    /// - When success, the `other` will become a empty [`RpcBuilder`].
    /// - All options including `cluster`, `commiment` and `program_id` will still be
    ///   the same of `self` after merging.
    pub fn try_merge(&mut self, other: &mut Self) -> crate::Result<()> {
        if self.payer != other.payer {
            return Err(crate::Error::invalid_argument("payer mismatched"));
        }

        // Push the rpc ix before merging.
        if let Some(ix) = self.take_rpc() {
            self.pre_instructions.push(ix);
        }

        // Merge ixs.
        self.pre_instructions.append(&mut other.pre_instructions);
        if let Some(ix) = other.take_rpc() {
            self.pre_instructions.push(ix);
        }

        // Merge signers.
        self.signers.append(&mut other.signers);

        // Merge compute budget.
        self.compute_budget += std::mem::take(&mut other.compute_budget);
        Ok(())
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone, T> RpcBuilder<'a, C, T> {
    pub fn payer(mut self, payer: C) -> Self {
        self.payer = payer.pubkey();
        self.builder = self.builder.payer(payer);
        self
    }

    /// Set cluster.
    pub fn cluster(mut self, url: &str) -> Self {
        self.builder = self.builder.cluster(url);
        self
    }

    /// Set commiment options.
    pub fn options(mut self, options: CommitmentConfig) -> Self {
        self.builder = self.builder.options(options);
        self
    }

    /// Add a signer to the signer list.
    pub fn signer(mut self, signer: &'a dyn Signer) -> Self {
        self.signers.push(signer);
        self
    }

    /// Set program id.
    pub fn program(mut self, program_id: Pubkey) -> Self {
        self.program_id = program_id;
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
    pub fn compute_budget(mut self, budget: ComputeBudget) -> Self {
        self.compute_budget = budget;
        self
    }

    /// Get mutable reference to the compute budget.
    pub fn compute_budget_mut(&mut self) -> &mut ComputeBudget {
        &mut self.compute_budget
    }

    fn get_compute_budget_instructions(
        &self,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> Vec<Instruction> {
        self.compute_budget
            .compute_budget_instructions(compute_unit_price_micro_lamports)
    }

    fn get_rpc_instruction(&self) -> Option<Instruction> {
        let ix_data = self.instruction_data.as_ref()?;
        Some(Instruction {
            program_id: self.program_id,
            data: ix_data.clone(),
            accounts: self.accounts.clone(),
        })
    }

    /// Get all instructions.
    pub fn instructions(&self) -> Vec<Instruction> {
        self.instructions_with_options(false, None)
    }

    /// Get all instructions with options.
    pub fn instructions_with_options(
        &self,
        without_compute_budget: bool,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> Vec<Instruction> {
        let mut instructions = if without_compute_budget {
            Vec::default()
        } else {
            self.get_compute_budget_instructions(compute_unit_price_micro_lamports)
        };
        instructions.append(&mut self.pre_instructions.clone());
        if let Some(ix) = self.get_rpc_instruction() {
            instructions.push(ix);
        }
        instructions
    }

    /// Get the output.
    pub fn output(&self) -> &T {
        &self.output
    }

    /// Set the output and return the previous.
    pub fn swap_output<U>(self, output: U) -> (RpcBuilder<'a, C, U>, T) {
        let Self {
            payer,
            signers,
            output: previous,
            program_id,
            builder,
            pre_instructions,
            accounts,
            instruction_data,
            compute_budget,
        } = self;

        (
            RpcBuilder {
                payer,
                signers,
                output,
                program_id,
                builder,
                pre_instructions,
                accounts,
                instruction_data,
                compute_budget,
            },
            previous,
        )
    }

    /// Set the output.
    pub fn with_output<U>(self, output: U) -> RpcBuilder<'a, C, U> {
        self.swap_output(output).0
    }

    /// Clear the output.
    pub fn clear_output(self) -> RpcBuilder<'a, C, ()> {
        self.swap_output(()).0
    }

    /// Insert an instruction before the rpc method.
    pub fn pre_instruction(mut self, ix: Instruction) -> Self {
        self.pre_instructions.push(ix);
        self
    }

    /// Insert instructions before the rpc method.
    pub fn pre_instructions(mut self, mut ixs: Vec<Instruction>) -> Self {
        self.pre_instructions.append(&mut ixs);
        self
    }

    /// Build and send the RPC request.
    pub async fn send(self) -> crate::Result<Signature> {
        let signature = self.build().send().await?;
        Ok(signature)
    }

    /// Build [`RequestBuilder`](anchor_client::RequestBuilder).
    pub fn build(self) -> anchor_client::RequestBuilder<'a, C> {
        self.build_with_options(false, None).0
    }

    /// Build [`RequestBuilder`](anchor_client::RequestBuilder) without compute budget.
    pub fn build_without_compute_budget(self) -> anchor_client::RequestBuilder<'a, C> {
        self.build_with_options(true, None).0
    }

    /// Build and output.
    pub fn build_with_options(
        self,
        without_compute_budget: bool,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> (anchor_client::RequestBuilder<'a, C>, T) {
        debug_assert!(
            self.builder.instructions().unwrap().is_empty(),
            "non-empty builder"
        );
        let request = self
            .instructions_with_options(without_compute_budget, compute_unit_price_micro_lamports)
            .into_iter()
            .fold(self.builder.program(self.program_id), |acc, ix| {
                acc.instruction(ix)
            });
        let request = self
            .signers
            .into_iter()
            .fold(request, |acc, signer| acc.signer(signer));
        (request, self.output)
    }

    /// Estimated the size of the result transaction.
    ///
    /// See [`transaction_size()`] for more information.
    pub fn transaction_size(&self, is_versioned_transaction: bool) -> usize {
        transaction_size(&self.instructions(), is_versioned_transaction)
    }

    /// Estimated the execution fee of the result transaction.
    pub async fn estimate_execution_fee(
        &self,
        client: &RpcClient,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> crate::Result<u64> {
        use anchor_client::solana_sdk::message::Message;

        let ixs = self.instructions_with_options(false, compute_unit_price_micro_lamports);
        let blockhash = client
            .get_latest_blockhash()
            .await
            .map_err(anchor_client::ClientError::from)?;
        let message = Message::new_with_blockhash(&ixs, Some(&self.payer), &blockhash);
        let fee = client
            .get_fee_for_message(&message)
            .await
            .map_err(anchor_client::ClientError::from)?;
        Ok(fee)
    }
}
