//! Provides the [`Module`] struct.

use crate::interpreter::{value::Value, Interpreter};
use crate::loader;
use crate::runtime;
use crate::runtime::resolver;
use std::fmt::{Debug, Formatter};

enum ModuleResolver<'env> {
    Borrowed(&'env dyn resolver::Resolver),
    Owned(resolver::BoxedResolver),
}

impl<'env> ModuleResolver<'env> {
    pub fn as_dyn_resolver(&self) -> &dyn resolver::Resolver {
        match self {
            Self::Borrowed(borrowed) => *borrowed,
            Self::Owned(owned) => owned.as_ref(),
        }
    }
}

/// Encapsulates all runtime state associated with a given IL4IL Module.
pub struct Module<'env> {
    runtime: &'env runtime::Runtime<'env>,
    module: loader::module::Module<'env>,
    resolver: ModuleResolver<'env>,
}

impl<'env> Module<'env> {
    pub(super) fn new(
        runtime: &'env runtime::Runtime<'env>,
        module: loader::module::Module<'env>,
        resolver: Option<resolver::BoxedResolver>,
    ) -> Self {
        Self {
            runtime,
            module,
            resolver: resolver
                .map(ModuleResolver::Owned)
                .unwrap_or_else(|| ModuleResolver::Borrowed(runtime.default_resolver())),
        }
    }

    pub fn runtime(&'env self) -> &'env runtime::Runtime<'env> {
        self.runtime
    }

    pub fn resolver(&'env self) -> &'env dyn resolver::Resolver {
        self.resolver.as_dyn_resolver()
    }

    pub fn module(&'env self) -> &'env loader::module::Module<'env> {
        &self.module
    }

    fn setup_interpreter(&'env self, instantiation: &'env loader::function::Instantiation<'env>, arguments: Box<[Value]>) -> Interpreter {
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
        arguments: Box<[Value]>,
    ) -> Interpreter {
        self.setup_interpreter(&self.module.function_instantiations()[usize::from(index)], arguments)
    }

    /// Initializes an interpreter for the module's entry point function, returning `None` if no entry point exists.
    ///
    /// See [`Interpreter::initialize`] for more information.
    ///
    /// [`Interpreter::initialize`]: crate::interpreter::Interpreter::initialize
    pub fn interpret_entry_point(&'env self, arguments: Box<[Value]>) -> Option<Interpreter> {
        self.module.entry_point().map(|entry| self.setup_interpreter(entry, arguments))
    }
}

impl Debug for Module<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module").field("module", &self.module).finish_non_exhaustive()
    }
}
