//! Module for interacting with the IL4IL type system.

use crate::module::Module;
use il4il::type_system;

#[derive(Debug, Eq, PartialEq)]
pub enum TypeKind {
    Integer(type_system::Integer),
    Float(type_system::Float),
}

pub struct Type<'env> {
    module: &'env Module<'env>,
    kind: lazy_init::LazyTransform<type_system::Type, TypeKind>,
}

impl<'env> Type<'env> {
    pub(crate) fn new(module: &'env Module<'env>, kind: type_system::Type) -> Self {
        Self {
            module,
            kind: lazy_init::LazyTransform::new(kind),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn kind(&'env self) -> &'env TypeKind {
        self.kind.get_or_create(|kind| match kind {
            type_system::Type::Integer(i) => TypeKind::Integer(i),
            type_system::Type::Float(f) => TypeKind::Float(f),
            _ => todo!("unsupported type"),
        })
    }

    /// Gets the size, in bits, of values of this type.
    pub fn bit_width(&'env self) -> std::num::NonZeroU32 {
        match self.kind() {
            TypeKind::Integer(type_system::Integer::Sized(i)) => i.bit_width().into(),
            TypeKind::Integer(type_system::Integer::Address(_)) => self.module.environment().address_size.size().bit_width().into(),
            TypeKind::Float(f) => f.bit_width().into(),
        }
    }
}

impl<'env> PartialEq for &'env Type<'env> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.module as *const _, other.module as *const _) && self.kind() == other.kind()
    }
}
