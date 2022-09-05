//! Provides a high-level representation of the IL4IL interpreter, providing support for debugging IL4IL bytecode and for executing
//! bytecode in multiple threads.
//!
//! For more information, see the documentation for [`Host`].

use crate::runtime::{configuration::Configuration, Runtime};

pub type HostScope<'host, 'parent> = &'host std::thread::Scope<'host, 'parent>;

pub struct Host<'host, 'parent: 'host> {
    runtime: Runtime<'host>,
    scope: HostScope<'host, 'parent>,
    //interpreters: Vec<Mutex>,
}

impl<'host, 'parent: 'host> Host<'host, 'parent> {
    pub fn with_configuration_in_scope(configuration: Configuration, scope: HostScope<'host, 'parent>) -> Self {
        Self {
            runtime: Runtime::with_configuration(configuration),
            scope,
        }
    }
}

impl std::fmt::Debug for Host<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host").field("scope", &self.scope).finish_non_exhaustive()
    }
}
