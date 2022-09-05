//! Provides the [`Module`] struct.

use crate::interpreter::{self, Interpreter};
use crate::loader;
use crate::runtime;
use std::fmt::{Debug, Formatter};

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

    fn setup_interpreter(
        &'env self,
        instantiation: &'env loader::function::Instantiation<'env>,
        arguments: Box<[interpreter::Value]>,
    ) -> Interpreter {
        Interpreter::initialize(self.runtime, instantiation, arguments)
    }

    /// Initializes an interpreter for a given function instantiation.
    ///
    /// See [`Interpreter::initialize`] for more information.
    ///
    /// [`Interpreter::initialize`]: crate::interpreter::Interpreter::initialize
    pub fn interpret_function_instantiation(
        &'env self,
        index: il4il::index::FunctionInstantiation,
        arguments: Box<[interpreter::Value]>,
    ) -> Interpreter {
        self.setup_interpreter(&self.module.function_instantiations()[usize::from(index)], arguments)
    }

    /// Initializes an interpreter for the module's entry point function, returning `None` if no entry point exists.
    ///
    /// See [`Interpreter::initialize`] for more information.
    ///
    /// [`Interpreter::initialize`]: crate::interpreter::Interpreter::initialize
    pub fn interpret_entry_point(&'env self, arguments: Box<[interpreter::Value]>) -> Option<Interpreter> {
        self.module.entry_point().map(|entry| self.setup_interpreter(entry, arguments))
    }
}

impl Debug for Module<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module").field("module", &self.module).finish_non_exhaustive()
    }
}
