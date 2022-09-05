//! Provides a high-level representation of the IL4IL interpreter, providing support for debugging IL4IL bytecode and for executing
//! bytecode in multiple threads.
//!
//! For more information, see the documentation for [`Host`].

mod module;
mod thread;

pub use module::HostModule;
pub use thread::InterpreterThread;

use crate::runtime::configuration::Configuration;
use crate::runtime::Runtime;

pub type HostScope<'host, 'parent> = &'host std::thread::Scope<'host, 'parent>;

/// Encapsulates all runtime state.
///
/// For more information, see the [`host`] module documentation.
///
/// [`host`]: crate::host
#[derive(Debug)]
pub struct Host<'host, 'parent: 'host> {
    runtime: Runtime<'host>,
    scope: HostScope<'host, 'parent>,
    //interpreters: Vec<Mutex>,
}

impl<'host, 'parent: 'host> Host<'host, 'parent> {
    /// Initializes the host with the given runtime configuration and [`std::thread::Scope`] in which interpreter threads are spawned in.
    ///
    /// # Example
    ///
    /// ```
    /// use il4il_vm::host::Host;
    /// use il4il_vm::runtime::configuration::Configuration;
    ///
    /// std::thread::scope(|scope| {
    ///     Host::with_configuration_in_scope(Configuration::HOST, scope);
    ///     // Start using the host to do things after this.
    /// });
    /// ```
    pub fn with_configuration_in_scope(configuration: Configuration, scope: HostScope<'host, 'parent>) -> Self {
        Self {
            runtime: Runtime::with_configuration(configuration),
            scope,
        }
    }

    /// Gets the scope used when spawning interpreter threads.
    pub fn scope(&'host self) -> HostScope<'host, 'parent> {
        self.scope
    }

    pub fn load_module(&'host self, module: il4il::validation::ValidModule<'parent>) -> HostModule<'host, 'parent> {
        HostModule::new(self, module)
    }
}
