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

// TODO: Rename to ReferenceKind, add a struct Reference
pub enum ReferenceKind<'env> {
    Inlined(Type<'env>),
    Indexed(&'env Type<'env>),
}

impl<'env> ReferenceKind<'env> {
    pub fn as_type(&self) -> &Type<'env> {
        match self {
            Self::Inlined(inlined) => inlined,
            Self::Indexed(indexed) => indexed,
        }
    }
}

type ReferenceInner<'env> = lazy_init::LazyTransform<(&'env Module<'env>, type_system::Reference), ReferenceKind<'env>>;

pub struct Reference<'env>(ReferenceInner<'env>);

impl<'env> Reference<'env> {
    pub(crate) fn new(module: &'env Module<'env>, ty: type_system::Reference) -> Self {
        Self(ReferenceInner::new((module, ty)))
    }

    pub fn kind(&self) -> &ReferenceKind<'env> {
        self.0.get_or_create(|(module, ty)| match ty {
            type_system::Reference::Inline(inlined) => ReferenceKind::Inlined(Type::new(module, inlined)),
            type_system::Reference::Index(index) => ReferenceKind::Indexed(&module.types()[usize::from(index)]),
        })
    }

    pub fn as_type(&self) -> &Type<'env> {
        self.kind().as_type()
    }
}

type ReferenceListInner<'env> = lazy_init::LazyTransform<(&'env Module<'env>, Box<[type_system::Reference]>), Box<[Reference<'env>]>>;

pub(crate) struct ReferenceList<'env>(ReferenceListInner<'env>);

impl<'env> ReferenceList<'env> {
    pub(crate) fn new(module: &'env Module<'env>, types: Box<[type_system::Reference]>) -> Self {
        Self(ReferenceListInner::new((module, types)))
    }

    pub fn types(&self) -> &[Reference<'env>] {
        self.0
            .get_or_create(|(module, references)| references.into_vec().into_iter().map(|r| Reference::new(module, r)).collect())
    }
}
