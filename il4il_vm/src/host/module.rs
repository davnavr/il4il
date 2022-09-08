//! Contains the [`HostModule`] struct.

use crate::host::{self, Host};
use crate::runtime;

/// Represents an IL4IL [`Module`] within a [`Host`].
///
/// [`Module`]: crate::runtime::Module
/// [`Host`]: crate::host::Host
pub struct HostModule<'host, 'parent: 'host> {
    host: &'host Host<'host, 'parent>,
    module: std::sync::Arc<runtime::Module<'host>>,
}

impl<'host, 'parent: 'host> HostModule<'host, 'parent> {
    pub(super) fn new(host: &'host Host<'host, 'parent>, module: il4il::validation::ValidModule<'parent>) -> Self {
        #[allow(unreachable_code)]
        Self {
            host,
            module: todo!("load module {module:?}"), // host.runtime.load_module(module),
        }
    }

    pub fn host(&'host self) -> &'host Host<'host, 'parent> {
        self.host
    }

    pub fn module(&'host self) -> &'host crate::loader::module::Module<'host> {
        self.module.module()
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
    ) -> Option<std::io::Result<host::InterpreterThread<'host, 'parent>>> {
        let entry_point: &'host crate::interpreter::Function<'host> = self.module().entry_point()?;
        Some(host::InterpreterThread::new(self.host, builder, entry_point, arguments))
    }
}
