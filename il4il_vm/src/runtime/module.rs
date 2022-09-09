//! Provides the [`Module`] struct.

use crate::interpreter::{value::Value, Interpreter};
use crate::loader;
use crate::runtime;
use crate::runtime::resolver;
use std::collections::hash_map;
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;

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

type FunctionImplementations<'env> =
    Mutex<rustc_hash::FxHashMap<il4il::index::FunctionTemplate, Box<runtime::function::FunctionImplementation<'env>>>>;

/// Encapsulates all runtime state associated with a given IL4IL Module.
pub struct Module<'env> {
    runtime: &'env runtime::Runtime<'env>,
    module: loader::module::Module<'env>,
    resolver: ModuleResolver<'env>,
    function_implementations: FunctionImplementations<'env>,
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
            function_implementations: Default::default(),
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

    pub fn get_function_implementation(
        &'env self,
        index: il4il::index::FunctionTemplate,
    ) -> Result<&'env runtime::function::FunctionImplementation<'env>, resolver::ImportError> {
        let mut implementations = self.function_implementations.lock().unwrap();
        let occupied_entry;
        let implementation = match implementations.entry(index) {
            hash_map::Entry::Occupied(occupied) => {
                occupied_entry = occupied;
                occupied_entry.get()
            }
            hash_map::Entry::Vacant(vacant) => vacant.insert(todo!()),
        };

        Ok(unsafe {
            // Safety: Box means pointer to the function implementation is always valid.
            &*(implementation.as_ref() as *const runtime::function::FunctionImplementation<'env>)
        })
    }

    pub fn get_function(
        &'env self,
        index: il4il::index::FunctionInstantiation,
    ) -> Result<runtime::function::Function<'env>, resolver::ImportError> {
        runtime::function::Function::new(self, &self.module.function_instantiations()[usize::from(index)])
    }

    pub fn get_entry_point_function(&'env self) -> Result<Option<runtime::function::Function<'env>>, resolver::ImportError> {
        Ok(if let Some(entry_point) = self.module.entry_point() {
            Some(runtime::function::Function::new(self, entry_point)?)
        } else {
            None
        })
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
    ) -> Result<Interpreter, resolver::ImportError> {
        Ok(Interpreter::initialize(self.runtime, self.get_function(index)?, arguments))
    }

    /// Initializes an interpreter for the module's entry point function, returning `None` if no entry point exists.
    ///
    /// See [`Interpreter::initialize`] for more information.
    ///
    /// [`Interpreter::initialize`]: crate::interpreter::Interpreter::initialize
    pub fn interpret_entry_point(&'env self, arguments: Box<[Value]>) -> Result<Option<Interpreter<'env>>, resolver::ImportError> {
        self.get_entry_point_function()
            .map(|entry_point| entry_point.map(|function| Interpreter::initialize(self.runtime, function, arguments)))
    }
}

impl Debug for Module<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module").field("module", &self.module).finish_non_exhaustive()
    }
}
