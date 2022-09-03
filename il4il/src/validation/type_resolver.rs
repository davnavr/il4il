//! Provides the [`IntoType`] trait.

use crate::type_system;
use crate::validation::{index_checker, ModuleContents};
use error_stack::Result;

pub(crate) trait IntoType {
    type Error: error_stack::Context;

    fn into_type(self, contents: &ModuleContents) -> Result<type_system::Type, Self::Error>;
}

impl IntoType for type_system::Type {
    type Error = std::convert::Infallible;

    fn into_type(self, _: &ModuleContents) -> Result<type_system::Type, Self::Error> {
        Ok(self)
    }
}

impl IntoType for &type_system::Type {
    type Error = std::convert::Infallible;

    fn into_type(self, _: &ModuleContents) -> Result<type_system::Type, Self::Error> {
        Ok(*self)
    }
}

impl IntoType for &type_system::Reference {
    type Error = index_checker::InvalidIndexError;

    fn into_type(self, contents: &ModuleContents) -> Result<type_system::Type, Self::Error> {
        match self {
            type_system::Reference::Inline(ty) => Ok(*ty),
            type_system::Reference::Index(index) => index_checker::get_type(*index, contents).copied(),
        }
    }
}

pub(crate) fn resolve_many<'a, T, I>(
    types: I,
    buffer: &'a mut Vec<type_system::Type>,
    contents: &ModuleContents,
) -> Result<&'a [type_system::Type], T::Error>
where
    T: IntoType,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let iterator = types.into_iter();
    buffer.clear();
    buffer.reserve_exact(iterator.len());
    for ty in iterator {
        buffer.push(ty.into_type(contents)?);
    }
    Ok(buffer.as_slice())
}
