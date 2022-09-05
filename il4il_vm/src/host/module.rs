//! Contains the [`HostModule`] struct.

use crate::host::Host;
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
        Self {
            host,
            module: host.runtime.load_module(module),
        }
    }

    pub fn host(&'host self) -> &'host Host<'host, 'parent> {
        self.host
    }
}
