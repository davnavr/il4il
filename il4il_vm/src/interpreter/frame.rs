//! Module for interacting with the IL4IL interpreter call stack.

use crate::interpreter::Value;
use crate::loader::function;
use il4il::index;

/// Represents a frame in the call stack.
pub struct Frame<'env> {
    function: &'env function::Instantiation<'env>,
    //block: &'env function::,
    block_index: index::Block,
    arguments: Box<[Value]>,
}

impl<'env> Frame<'env> {
    pub(super) fn new(function: &'env function::Instantiation<'env>, arguments: Box<[Value]>) -> Self {
        Self {
            function,
            block_index: index::Block::new(0),
            arguments,
        }
    }

    pub fn function(&self) -> &'env function::Instantiation {
        self.function
    }

    pub fn block_index(&self) -> index::Block {
        self.block_index
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("function", self.function)
            .field("block_index", &self.block_index)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}
