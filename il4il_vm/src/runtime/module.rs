//! Provides the [`Module`] struct.

use crate::interpreter::Interpreter;
use crate::loader;
use crate::runtime;

/// Encapsulates all runtime state associated with a given IL4IL Module.
pub struct Module<'env> {
    runtime: &'env runtime::Runtime<'env>,
    module: loader::module::Module<'env>,
}

impl<'env> Module<'env> {
    pub(super) fn new(runtime: &'env runtime::Runtime<'env>, module: loader::module::Module<'env>) -> Self {
        Self { runtime, module }
    }

    pub fn runtime(&'env self) -> &'env runtime::Runtime<'env> {
        self.runtime
    }

    pub fn module(&'env self) -> &'env loader::module::Module<'env> {
        &self.module
    }

    fn setup_interpreter(&'env self, instantiation: &'env loader::function::Instantiation<'env>) -> Interpreter {
        Interpreter::initialize(self.runtime, instantiation)
    }

    pub fn interpret_function_instantiation(&'env self, index: il4il::index::FunctionInstantiation) -> Interpreter {
        self.setup_interpreter(&self.module.function_instantiations()[usize::from(index)])
    }

    pub fn interpret_entry_point(&'env self) -> Option<Interpreter> {
        self.module.entry_point().map(|entry| self.setup_interpreter(entry))
    }
}
