use std::{borrow::Cow, collections::BTreeMap, ops::Deref};

use smallvec::SmallVec;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::NullSigner,
    signer::Signer,
    transaction::VersionedTransaction,
};

use crate::{
    address_lookup_table::AddressLookupTables, compute_budget::ComputeBudget,
    signer::BoxClonableSigner, transaction_group::TransactionGroupOptions,
};

const ATOMIC_SIZE: usize = 3;
const PARALLEL_SIZE: usize = 2;

/// A trait representing types that can be converted into [`AtomicGroup`]s.
pub trait IntoAtomicGroup {
    /// Hint.
    type Hint;

    /// Convert into [`AtomicGroup`]s.
    fn into_atomic_group(self, hint: &Self::Hint) -> crate::Result<AtomicGroup>;
}

/// Options for getting instructions.
#[derive(Debug, Clone, Default)]
pub struct GetInstructionsOptions {
    /// Without compute budget instruction.
    pub without_compute_budget: bool,
    /// Compute unit price in micro lamports.
    pub compute_unit_price_micro_lamports: Option<u64>,
    /// If set, a memo will be included in the final transaction.
    pub memo: Option<String>,
}

/// A group of instructions that are expected to be executed in the same transaction.
#[derive(Debug, Clone)]
pub struct AtomicGroup {
    payer: Pubkey,
    signers: BTreeMap<Pubkey, NullSigner>,
    owned_signers: BTreeMap<Pubkey, BoxClonableSigner<'static>>,
    instructions: SmallVec<[Instruction; ATOMIC_SIZE]>,
    compute_budget: ComputeBudget,
}

impl AtomicGroup {
    /// Create from an iterator of instructions.
    pub fn with_instructions(
        payer: &Pubkey,
        instructions: impl IntoIterator<Item = Instruction>,
    ) -> Self {
        Self {
            payer: *payer,
            signers: BTreeMap::from([(*payer, NullSigner::new(payer))]),
            owned_signers: Default::default(),
            instructions: SmallVec::from_iter(instructions),
            compute_budget: Default::default(),
        }
    }

    /// Create a new empty group.
    pub fn new(payer: &Pubkey) -> Self {
        Self::with_instructions(payer, None)
    }

    /// Add an instruction.
    pub fn add(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    /// Add a signer.
    pub fn add_signer(&mut self, signer: &Pubkey) -> &mut Self {
        self.signers.insert(*signer, NullSigner::new(signer));
        self
    }

    /// Add an owned signer.
    pub fn add_owned_signer(&mut self, signer: impl Signer + Clone + 'static) -> &mut Self {
        self.owned_signers
            .insert(signer.pubkey(), BoxClonableSigner::new(signer));
        self
    }

    /// Get compute budget.
    pub fn compute_budget(&self) -> &ComputeBudget {
        &self.compute_budget
    }

    /// Get mutable reference to the compute budget.
    pub fn compute_budget_mut(&mut self) -> &mut ComputeBudget {
        &mut self.compute_budget
    }

    /// Returns the pubkey of the payer.
    pub fn payer(&self) -> &Pubkey {
        &self.payer
    }

    /// Returns signers that need to be provided externally including the payer.
    pub fn external_signers(&self) -> impl Iterator<Item = &Pubkey> + '_ {
        self.signers.keys()
    }

    fn compute_budget_instructions(
        &self,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> Vec<Instruction> {
        self.compute_budget
            .compute_budget_instructions(compute_unit_price_micro_lamports)
    }

    /// Returns instructions.
    pub fn instructions_with_options(
        &self,
        options: GetInstructionsOptions,
    ) -> impl Iterator<Item = Cow<'_, Instruction>> {
        let compute_budget_instructions = if options.without_compute_budget {
            Vec::default()
        } else {
            self.compute_budget_instructions(options.compute_unit_price_micro_lamports)
        };
        let memo_instruction = options
            .memo
            .as_ref()
            .map(|s| spl_memo::build_memo(s.as_bytes(), &[&self.payer]));
        compute_budget_instructions
            .into_iter()
            .chain(memo_instruction)
            .map(Cow::Owned)
            .chain(self.instructions.iter().map(Cow::Borrowed))
    }

    /// Estimates the transaciton size.
    pub fn transaction_size(
        &self,
        is_versioned_transaction: bool,
        luts: Option<&AddressLookupTables>,
        options: GetInstructionsOptions,
    ) -> usize {
        let addresses = luts.as_ref().map(|luts| luts.addresses());
        crate::utils::transaction_size(
            self.payer,
            &self.instructions_with_options(options).collect::<Vec<_>>(),
            is_versioned_transaction,
            addresses.as_ref(),
            luts.as_ref().map(|luts| luts.len()).unwrap_or_default(),
        )
    }

    /// Estimates the transaction size after merge.
    pub fn transaction_size_after_merge(
        &self,
        other: &Self,
        is_versioned_transaction: bool,
        luts: Option<&AddressLookupTables>,
        options: GetInstructionsOptions,
    ) -> usize {
        let addresses = luts.as_ref().map(|luts| luts.addresses());
        crate::utils::transaction_size(
            self.payer,
            &self
                .instructions_with_options(options)
                .chain(other.instructions_with_options(GetInstructionsOptions {
                    without_compute_budget: true,
                    ..Default::default()
                }))
                .collect::<Vec<_>>(),
            is_versioned_transaction,
            addresses.as_ref(),
            luts.as_ref().map(|luts| luts.len()).unwrap_or_default(),
        )
    }

    /// Merge two [`AtomicGroup`]s.
    ///
    /// # Note
    /// - Merging does not change the payer of the current [`AtomicGroup`].
    pub fn merge(&mut self, mut other: Self) -> &mut Self {
        self.signers.append(&mut other.signers);
        self.owned_signers.append(&mut other.owned_signers);
        self.instructions.extend(other.instructions);
        self.compute_budget += other.compute_budget;
        self
    }

    fn v0_message_with_blockhash_and_options(
        &self,
        recent_blockhash: Hash,
        options: GetInstructionsOptions,
        luts: Option<&AddressLookupTables>,
    ) -> crate::Result<v0::Message> {
        let instructions = self
            .instructions_with_options(options)
            .map(|ix| (*ix).clone())
            .collect::<Vec<_>>();
        let luts = luts
            .map(|t| t.accounts().collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(v0::Message::try_compile(
            self.payer(),
            &instructions,
            &luts,
            recent_blockhash,
        )?)
    }

    /// Create versioned message with the given blockhash and options.
    pub fn message_with_blockhash_and_options(
        &self,
        recent_blockhash: Hash,
        options: GetInstructionsOptions,
        luts: Option<&AddressLookupTables>,
    ) -> crate::Result<VersionedMessage> {
        Ok(VersionedMessage::V0(
            self.v0_message_with_blockhash_and_options(recent_blockhash, options, luts)?,
        ))
    }

    /// Create partially signed transaction with the given blockhash and options.
    pub fn partially_signed_transaction_with_blockhash_and_options(
        &self,
        recent_blockhash: Hash,
        options: GetInstructionsOptions,
        luts: Option<&AddressLookupTables>,
    ) -> crate::Result<VersionedTransaction> {
        let message = self.message_with_blockhash_and_options(recent_blockhash, options, luts)?;
        let signers = self
            .signers
            .values()
            .map(|s| s as &dyn Signer)
            .chain(self.owned_signers.values().map(|s| s as &dyn Signer))
            .collect::<Vec<_>>();
        Ok(VersionedTransaction::try_new(message, &signers)?)
    }
}

impl Extend<Instruction> for AtomicGroup {
    fn extend<T: IntoIterator<Item = Instruction>>(&mut self, iter: T) {
        self.instructions.extend(iter);
    }
}

impl Deref for AtomicGroup {
    type Target = [Instruction];

    fn deref(&self) -> &Self::Target {
        self.instructions.deref()
    }
}

/// A group of atomic instructions that can be executed in parallel.
#[derive(Debug, Clone, Default)]
pub struct ParallelGroup(SmallVec<[AtomicGroup; PARALLEL_SIZE]>);

impl ParallelGroup {
    /// Add an [`AtomicGroup`].
    pub fn add(&mut self, group: AtomicGroup) -> &mut Self {
        self.0.push(group);
        self
    }

    pub(crate) fn optimize(
        &mut self,
        options: &TransactionGroupOptions,
        luts: &AddressLookupTables,
        allow_payer_change: bool,
    ) -> &mut Self {
        if options.optimize(&mut self.0, luts, allow_payer_change) {
            self.0 = self.0.drain(..).filter(|group| !group.is_empty()).collect();
        }
        self
    }

    pub(crate) fn single(&self) -> Option<&AtomicGroup> {
        if self.0.len() == 1 {
            Some(&self.0[0])
        } else {
            None
        }
    }

    pub(crate) fn single_mut(&mut self) -> Option<&mut AtomicGroup> {
        if self.0.len() == 1 {
            Some(&mut self.0[0])
        } else {
            None
        }
    }

    pub(crate) fn into_single(mut self) -> Option<AtomicGroup> {
        if self.0.len() == 1 {
            Some(self.0.remove(0))
        } else {
            None
        }
    }
}

impl From<AtomicGroup> for ParallelGroup {
    fn from(value: AtomicGroup) -> Self {
        let mut this = Self::default();
        this.add(value);
        this
    }
}

impl FromIterator<AtomicGroup> for ParallelGroup {
    fn from_iter<T: IntoIterator<Item = AtomicGroup>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl Deref for ParallelGroup {
    type Target = [AtomicGroup];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
