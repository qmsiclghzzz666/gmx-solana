use std::ops::Deref;

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

/// A wrapper of [`RequestBuilder`](anchor_client::RequestBuilder)
/// better instruction insertion methods.
#[must_use]
pub struct RpcBuilder<'a, C, T = ()> {
    output: T,
    program_id: Pubkey,
    builder: anchor_client::RequestBuilder<'a, C>,
    pre_instructions: Vec<Instruction>,
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
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RpcBuilder<'a, C> {
    /// Create a new [`RpcBuilder`] from [`Program`].
    pub fn new(program: &'a Program<C>) -> Self {
        Self {
            output: (),
            program_id: program.id(),
            builder: program.request(),
            pre_instructions: Default::default(),
            accounts: Default::default(),
            instruction_data: None,
            post_instructions: Default::default(),
            compute_budget: Some(ComputeBudget::default()),
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone, T> RpcBuilder<'a, C, T> {
    /// Set payer.
    pub fn payer(mut self, payer: C) -> Self {
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
        self.builder = self.builder.signer(signer);
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
        instructions.append(&mut self.post_instructions.clone());
        instructions
    }

    /// Get the output.
    pub fn output(&self) -> &T {
        &self.output
    }

    /// Set the output.
    pub fn with_output<U>(self, output: U) -> RpcBuilder<'a, C, U> {
        let Self {
            program_id,
            builder,
            pre_instructions,
            accounts,
            instruction_data,
            post_instructions,
            compute_budget,
            ..
        } = self;

        RpcBuilder {
            output,
            program_id,
            builder,
            pre_instructions,
            accounts,
            instruction_data,
            post_instructions,
            compute_budget,
        }
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

    /// Insert an instruction after the rpc method.
    pub fn post_instruction(mut self, ix: Instruction) -> Self {
        self.post_instructions.push(ix);
        self
    }

    /// Insert instructions after the rpc method.
    pub fn post_instructions(mut self, mut ixs: Vec<Instruction>) -> Self {
        self.post_instructions.append(&mut ixs);
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
        (request, self.output)
    }
}
