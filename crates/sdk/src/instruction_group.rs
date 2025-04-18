use std::ops::Deref;

use smallvec::SmallVec;
use solana_sdk::instruction::Instruction;

const ATOMIC_SIZE: usize = 3;
const PARALLEL_SIZE: usize = 2;

/// A trait representing types that can be converted into [`AtomicInstructionGroup`]s.
pub trait IntoAtomicInstructionGroup {
    /// Hint.
    type Hint;

    /// Convert into [`AtomicInstructionGroup`]s.
    fn into_atomic_instruction_group(
        self,
        hint: &Self::Hint,
    ) -> crate::Result<AtomicInstructionGroup>;
}

/// A group of instructions that are expected to be executed in the same transaction.
#[derive(Debug, Clone, Default)]
pub struct AtomicInstructionGroup(SmallVec<[Instruction; ATOMIC_SIZE]>);

impl AtomicInstructionGroup {
    /// Add an instruction.
    pub fn add(&mut self, instruction: Instruction) -> &mut Self {
        self.0.push(instruction);
        self
    }
}

impl From<Instruction> for AtomicInstructionGroup {
    fn from(value: Instruction) -> Self {
        let mut this = Self::default();
        this.add(value);
        this
    }
}

impl FromIterator<Instruction> for AtomicInstructionGroup {
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl Extend<Instruction> for AtomicInstructionGroup {
    fn extend<T: IntoIterator<Item = Instruction>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl Deref for AtomicInstructionGroup {
    type Target = [Instruction];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// A group of atomic instructions that can be executed in parallel.
#[derive(Debug, Clone, Default)]
pub struct InstructionGroup(SmallVec<[AtomicInstructionGroup; PARALLEL_SIZE]>);

impl InstructionGroup {
    /// Add an atomic instrucions group.
    pub fn add(&mut self, group: AtomicInstructionGroup) -> &mut Self {
        self.0.push(group);
        self
    }
}

impl From<AtomicInstructionGroup> for InstructionGroup {
    fn from(value: AtomicInstructionGroup) -> Self {
        let mut this = Self::default();
        this.add(value);
        this
    }
}

impl FromIterator<AtomicInstructionGroup> for InstructionGroup {
    fn from_iter<T: IntoIterator<Item = AtomicInstructionGroup>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl Deref for InstructionGroup {
    type Target = [AtomicInstructionGroup];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
