//! Contains the [`HostModule`] struct.

use crate::host::{self, Host};
use crate::runtime;

/// Represents an IL4IL [`Module`] within a [`Host`].
///
/// [`Module`]: crate::runtime::Module
/// [`Host`]: crate::host::Host
pub struct HostModule<'host, 'scope: 'host, 'env: 'scope> {
    host: &'host Host<'host, 'scope, 'env>,
    module: &'env runtime::Module<'env>,
}

impl<'host, 'scope: 'host, 'env: 'host> HostModule<'host, 'scope, 'env> {
    pub(super) fn new(host: &'host Host<'host, 'scope, 'env>, module: il4il::validation::ValidModule<'env>) -> Self {
        Self {
            host,
            module: host.runtime.load_module(module, None),
        }
    }

    pub fn host(&'host self) -> &'host Host<'host, 'scope, 'env> {
        self.host
    }

    pub fn module(&'host self) -> &'env runtime::Module<'env> {
        self.module
    }

    /// Spawns an [`InterpreterThread`] to execute the module's entry point function.
    ///
    /// If no entry point is defined, returns `None`.
    ///
    /// [`InterpreterThread`]: crate::host::InterpreterThread
    pub fn interpret_entry_point(
        &'host self,
        builder: std::thread::Builder,
        arguments: Box<[crate::interpreter::value::Value]>,
    ) -> Option<std::io::Result<host::InterpreterThread<'host, 'scope, 'env>>> {
        // TODO: Handle an import error
        let entry_point: runtime::Function<'env> = self.module.get_entry_point_function().transpose()?.unwrap();
        Some(host::InterpreterThread::<'host, 'scope, 'env>::new(
            self.host,
            builder,
            entry_point,
            arguments,
        ))
    }
}

impl std::fmt::Debug for HostModule<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostModule").field("module", self.module).finish_non_exhaustive()
    }
}
