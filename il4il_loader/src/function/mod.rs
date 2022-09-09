//! Contains types representing IL4IL functions.

mod signature;

pub use signature::Signature;

pub mod template;

use crate::module::Module;
use std::fmt::{Debug, Formatter};

pub struct Instantiation<'env> {
    module: &'env Module<'env>,
    index: il4il::index::FunctionInstantiation,
    template: lazy_init::LazyTransform<il4il::index::FunctionTemplate, &'env template::Template<'env>>,
}

impl<'env> Instantiation<'env> {
    pub(crate) fn new(
        module: &'env Module<'env>,
        index: il4il::index::FunctionInstantiation,
        instantiation: il4il::function::Instantiation,
    ) -> Self {
        Self {
            module,
            index,
            template: lazy_init::LazyTransform::new(instantiation.template),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn index(&'env self) -> il4il::index::FunctionInstantiation {
        self.index
    }

    pub fn template(&'env self) -> &'env template::Template<'env> {
        self.template
            .get_or_create(|index| &self.module.function_templates()[usize::from(index)])
    }
}

impl Debug for Instantiation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instantiation")
            .field("template", &crate::debug::LazyDebug(&self.template))
            .finish_non_exhaustive()
    }
}
