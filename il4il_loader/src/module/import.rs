//! Contains the [`Import`] struct.

use crate::module::Module;
use il4il::identifier::Id;

/// Represents an imported module.
pub struct Import<'env> {
    importer: &'env Module<'env>,
    name: std::borrow::Cow<'env, Id>,
}

impl<'env> Import<'env> {
    pub(super) fn new(importer: &'env Module<'env>, import: il4il::module::ModuleName<'env>) -> Self {
        Self {
            importer,
            name: import.name,
        }
    }

    pub fn importer(&'env self) -> &'env Module<'env> {
        self.importer
    }

    pub fn name(&'env self) -> &'env Id {
        &self.name
    }
}
