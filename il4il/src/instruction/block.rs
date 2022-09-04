//! Provides the representation for a basic block.

use crate::instruction::Instruction;
use crate::type_system;

/// Reprsents a [basic block](https://en.wikipedia.org/wiki/Basic_block) in IL4IL, which is a sequence of instructions that must end with a
/// branch instruction.
///
/// Use [`Instruction::is_terminator`] to determine whether an [`Instruction`] can be used at the end of a block.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct Block {
    pub(crate) types: Box<[type_system::Reference]>, // Vec<>, another usize might not hurt, will make it align better.
    input_count: usize,
    pub instructions: Vec<Instruction>,
}

impl Block {
    /// Creates a code block with the specified input register, and temporary register types.
    ///
    /// # Panics
    ///
    /// Panics if the number of input registers exceeds the total number of types.
    pub fn from_types(types: Box<[type_system::Reference]>, input_count: usize, instructions: Vec<Instruction>) -> Self {
        assert!(types.len() >= input_count);
        Self {
            types,
            input_count,
            instructions,
        }
    }

    pub fn new<I, T>(input_types: I, temporary_types: T, instructions: Vec<Instruction>) -> Self
    where
        I: IntoIterator<Item = type_system::Reference>,
        I::IntoIter: ExactSizeIterator,
        T: IntoIterator<Item = type_system::Reference>,
    {
        let input_types_iter = input_types.into_iter();
        let input_count = input_types_iter.len();
        Self::from_types(
            input_types_iter.into_iter().chain(temporary_types).collect(),
            input_count,
            instructions,
        )
    }

    pub fn input_count(&self) -> usize {
        self.input_count
    }

    /// Gets the total number of temporary registers defined in this block.
    pub fn temporary_count(&self) -> usize {
        self.types.len() - self.input_count
    }

    pub fn input_types(&self) -> &[type_system::Reference] {
        &self.types[0..self.input_count]
    }

    pub fn temporary_types(&self) -> &[type_system::Reference] {
        &self.types[self.input_count..]
    }

    // TODO: have a TemporaryRegisters structure which is like a Vec, but only mutates the latter portion of self.types
}
