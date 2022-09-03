//! Provides the [`Module`] struct.

use crate::loader;
use crate::runtime;

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
}
