//! Provides a high-level representation of the IL4IL interpreter, providing support for debugging IL4IL bytecode and for executing
//! bytecode in multiple threads.
//!
//! For more information, see the documentation for [`Host`].

mod module;
mod thread;

pub use module::HostModule;
pub use thread::InterpreterThread;

pub mod debugger;

use crate::runtime::Runtime;

pub type HostScope<'scope, 'env> = &'scope std::thread::Scope<'scope, 'env>;

/// Encapsulates all runtime state.
///
/// For more information, see the [`host`] module documentation.
///
/// [`host`]: crate::host
pub struct Host<'host, 'scope: 'host, 'env: 'scope> {
    runtime: &'env Runtime<'env>,
    scope: HostScope<'scope, 'env>,
    // Allows fields to be added later that reference data owned by Host
    _phantom: std::marker::PhantomData<&'host ()>,
    //interpreters: Vec<Mutex>,
}

impl<'host, 'scope: 'host, 'env: 'scope> Host<'host, 'scope, 'env> {
    /// Initializes the host with a [`Runtime`] in which modules are stored and a [`std::thread::Scope`] in which interpreter threads are
    /// spawned in.
    ///
    /// # Example
    ///
    /// ```
    /// use il4il_vm::host::Host;
    /// use il4il_vm::runtime::Runtime;
    /// use il4il_vm::runtime::configuration::Configuration;
    ///
    /// let runtime = Runtime::new();
    ///
    /// std::thread::scope(|scope| {
    ///     Host::with_runtime(&runtime, scope);
    ///     // Start using the host to do things after this.
    /// });
    /// ```
    pub fn with_runtime(runtime: &'env Runtime<'env>, scope: HostScope<'scope, 'env>) -> Self {
        Self {
            runtime,
            scope,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Gets the scope used when spawning interpreter threads.
    pub fn scope(&'host self) -> HostScope<'scope, 'env> {
        self.scope
    }

    /// Initializes a given `module` for interpretation.
    ///
    /// To begin execution, call the [`HostModule::interpret_entry_point`] function.
    pub fn load_module(&'host self, module: il4il::validation::ValidModule<'env>) -> HostModule<'host, 'scope, 'env> {
        HostModule::new(self, module)
    }
}

impl std::fmt::Debug for Host<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("runtime", self.runtime)
            .field("scope", self.scope)
            .finish_non_exhaustive()
    }
}
