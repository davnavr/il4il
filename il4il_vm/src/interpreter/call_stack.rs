//! Module for interacting with the IL4IL interpreter call stack.

use crate::interpreter::value::Value;
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

/// A frame in the call stack corresponding to a function provided by the host.
#[derive(Debug)]
pub struct HostFrame<'env> {
    function: &'env crate::runtime::HostFunction<'env>,
}

/// A frame in the call stack corresponding to a function implemented in IL4IL bytecode.
pub struct BytecodeFrame<'env> {
    block: &'env code::Block<'env>,
    instruction_pointer: std::cell::RefCell<InstructionPointer<'env>>,
}

impl<'env> BytecodeFrame<'env> {
    fn from_definition(definition: &'env function::template::Definition<'env>) -> Self {
        let block = definition.body().entry_block();

        Self {
            block,
            instruction_pointer: std::cell::RefCell::new(InstructionPointer::new(block.instructions())),
        }
    }

    pub fn block(&self) -> &'env code::Block<'env> {
        self.block
    }

    pub fn block_index(&self) -> index::Block {
        self.block.index()
    }

    pub fn instruction_index(&self) -> usize {
        self.instruction_pointer.borrow().index()
    }

    pub(super) fn advance(&self) -> &'env Instruction {
        self.instruction_pointer
            .borrow_mut()
            .next()
            .expect("expected terminator instruction to be handled")
    }

    pub fn has_hit_breakpoint(&self, breakpoints: &crate::host::debugger::breakpoint::BreakpointLookup<'env>) -> bool {
        breakpoints
            .get(&crate::host::debugger::breakpoint::Location::new(
                self.block,
                self.instruction_index(),
            ))
            .is_some()
    }
}

impl std::fmt::Debug for BytecodeFrame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BytecodeFrame")
            .field("block_index", &self.block_index())
            .field("instruction_index", &self.instruction_index())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum FrameKind<'env> {
    Host(HostFrame<'env>),
    Bytecode(BytecodeFrame<'env>),
}

/// Represents a frame in the call stack.
pub struct Frame<'env> {
    runtime: &'env crate::runtime::Runtime<'env>,
    function: &'env function::Instantiation<'env>,
    arguments: Box<[Value]>,
    kind: FrameKind<'env>,
}

impl<'env> Frame<'env> {
    pub(super) fn new(
        runtime: &'env crate::runtime::Runtime<'env>,
        function: &'env function::Instantiation<'env>,
        arguments: Box<[Value]>,
    ) -> Self {
        Self {
            runtime,
            function,
            arguments,
            kind: match function.template().kind() {
                function::template::TemplateKind::Definition(definition) => FrameKind::Bytecode(BytecodeFrame::from_definition(definition)),
                function::template::TemplateKind::Import(import) => {
                    todo!()
                }
            },
        }
    }

    pub fn function(&self) -> &'env function::Instantiation {
        self.function
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }

    pub fn kind(&self) -> &FrameKind<'env> {
        &self.kind
    }

    pub(super) fn create_value(&self, value: &instruction::Value, value_type: &'env crate::loader::types::Type<'env>) -> Value {
        match value {
            instruction::Value::Constant(constant) => {
                Value::from_constant_value(constant, value_type, self.runtime.configuration().endianness)
            }
        }
    }
}

impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("function", self.function)
            .field("arguments", &self.arguments)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
