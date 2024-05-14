use std::ops::{AddAssign, Deref};

use anchor_client::{
    anchor_lang::{InstructionData, ToAccountMetas},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signer::Signer,
    },
    Program,
};

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
    /// the same of `self` after merging.
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
    /// the same of `self` after merging.
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

    fn get_compute_budget_instructions(&self) -> Vec<Instruction> {
        self.compute_budget.compute_budget_instructions()
    }

    /// Get all instructions.
    pub fn instructions(&self) -> Vec<Instruction> {
        let mut instructions = self.get_compute_budget_instructions();
        instructions.append(&mut self.pre_instructions.clone());
        if let Some(ix_data) = &self.instruction_data {
            instructions.push(Instruction {
                program_id: self.program_id,
                data: ix_data.clone(),
                accounts: self.accounts.clone(),
            });
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

    /// Build [`RequestBuilder`](anchor_client::RequestBuilder).
    pub fn build(self) -> anchor_client::RequestBuilder<'a, C> {
        self.build_with_output().0
    }

    /// Build and output.
    pub fn build_with_output(self) -> (anchor_client::RequestBuilder<'a, C>, T) {
        debug_assert!(
            self.builder.instructions().unwrap().is_empty(),
            "non-empty builder"
        );
        let request = self
            .instructions()
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
            limit_units: 50_000,
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

    /// Build compute budget instructions.
    pub fn compute_budget_instructions(&self) -> Vec<Instruction> {
        vec![
            ComputeBudgetInstruction::set_compute_unit_limit(self.limit_units),
            ComputeBudgetInstruction::set_compute_unit_price(self.price_micro_lamports),
        ]
    }
}

impl AddAssign for ComputeBudget {
    fn add_assign(&mut self, rhs: Self) {
        self.limit_units += rhs.limit_units;
        self.price_micro_lamports = self.price_micro_lamports.max(rhs.price_micro_lamports);
    }
}
