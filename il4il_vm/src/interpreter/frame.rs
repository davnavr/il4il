//! Module for interacting with the IL4IL interpreter call stack.

use crate::interpreter::Value;
use crate::loader::{code, function};
use il4il::index;
use il4il::instruction::{self, Instruction};

struct InstructionPointer<'env> {
    index: usize,
    instructions: std::slice::Iter<'env, Instruction>,
}

impl<'env> InstructionPointer<'env> {
    fn new(instructions: &'env [Instruction]) -> Self {
        Self {
            index: 0,
            instructions: instructions.iter(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'env> Iterator for InstructionPointer<'env> {
    type Item = &'env Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let instruction = self.instructions.next()?;
        self.index += 1;
        Some(instruction)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.instructions.size_hint()
    }
}

impl std::iter::ExactSizeIterator for InstructionPointer<'_> {
    fn len(&self) -> usize {
        self.instructions.len()
    }
}

/// Represents a frame in the call stack.
pub struct Frame<'env> {
    function: &'env function::Instantiation<'env>,
    block: &'env code::Block<'env>,
    arguments: Box<[Value]>,
    instruction_pointer: InstructionPointer<'env>,
}

impl<'env> Frame<'env> {
    pub(super) fn new(function: &'env function::Instantiation<'env>, arguments: Box<[Value]>) -> Self {
        let block = match function.template().kind() {
            function::template::TemplateKind::Definition(definition) => definition.body().entry_block(),
        };

        Self {
            function,
            block,
            arguments,
            instruction_pointer: InstructionPointer::new(block.instructions()),
        }
    }

    pub fn function(&self) -> &'env function::Instantiation {
        self.function
    }

    pub fn block(&self) -> &'env code::Block<'env> {
        self.block
    }

    pub fn block_index(&self) -> index::Block {
        self.block.index()
    }

    pub fn instruction_index(&self) -> usize {
        self.instruction_pointer.index()
    }

    pub(super) fn advance(&mut self) -> &'env Instruction {
        self.instruction_pointer
            .next()
            .expect("expected terminator instruction to be handled")
    }

    pub(super) fn create_value(&self, value: &instruction::Value, value_type: &'env crate::loader::types::Type<'env>) -> Value {
        match value {
            instruction::Value::Constant(constant) => Value::from_constant_value(constant, value_type),
        }
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("function", self.function)
            .field("block_index", &self.block_index())
            .field("instruction_index", &self.instruction_index())
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}
