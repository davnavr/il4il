//! Contains the [`Signature`] struct.

use crate::module::Module;
use crate::types;

type Inner<'env> = lazy_init::LazyTransform<il4il::function::Signature, (usize, types::ReferenceList<'env>)>;

/// Represents a function signature.
pub struct Signature<'env> {
    module: &'env Module<'env>,
    inner: Inner<'env>,
}

impl<'env> Signature<'env> {
    pub(crate) fn new(module: &'env Module<'env>, signature: il4il::function::Signature) -> Self {
        Self {
            module,
            inner: Inner::new(signature),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    fn all_types(&'env self) -> &'env (usize, types::ReferenceList<'env>) {
        self.inner.get_or_create(|signature| {
            (
                signature.result_type_count(),
                types::ReferenceList::new(self.module, signature.into_types()),
            )
        })
    }

    pub fn result_types(&'env self) -> &'env [types::Reference<'env>] {
        let (result_count, all_types) = self.all_types();
        &all_types.types()[0..*result_count]
    }

    pub fn parameter_types(&'env self) -> &'env [types::Reference<'env>] {
        let (result_count, all_types) = self.all_types();
        &all_types.types()[*result_count..]
    }
}
