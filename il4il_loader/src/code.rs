//! Representation of IL4IL function bodies.

use crate::module::Module;
use std::fmt::{Debug, Formatter};

pub struct Code<'env> {
    module: &'env Module<'env>,
    index: il4il::index::FunctionBody,
}

impl<'env> Code<'env> {
    pub(crate) fn new(module: &'env Module<'env>, index: il4il::index::FunctionBody, _code: il4il::function::Body) -> Self {
        Self { module, index }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn index(&'env self) -> il4il::index::FunctionBody {
        self.index
    }
}

impl Debug for Code<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code").finish_non_exhaustive()
    }
}
