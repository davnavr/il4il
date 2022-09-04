//! Module for interacting with the IL4IL interpreter call stack.

use crate::interpreter::Value;
use crate::loader::function;

/// Represents a frame in the call stack.
pub struct Frame<'env> {
    function: &'env function::Instantiation<'env>,
    arguments: Box<[Value]>,
}

impl<'env> Frame<'env> {
    pub(super) fn new(function: &'env function::Instantiation<'env>, arguments: Box<[Value]>) -> Self {
        Self { function, arguments }
    }

    pub fn function(&self) -> &'env function::Instantiation {
        self.function
    }

    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("function", self.function)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}
