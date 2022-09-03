//! Provides the [`IntoType`] trait.

use crate::type_system;
use crate::validation::{index_checker, ModuleContents};
use error_stack::Result;

pub(crate) trait IntoType<'a> {
    type Error: error_stack::Context;

    fn into_type(self, contents: &'a ModuleContents) -> Result<&'a type_system::Type, Self::Error>;
}

impl<'a> IntoType<'a> for &'a type_system::Type {
    type Error = std::convert::Infallible;

    fn into_type(self, _: &'a ModuleContents) -> Result<&'a type_system::Type, Self::Error> {
        Ok(self)
    }
}

impl<'a> IntoType<'a> for &'a type_system::Reference {
    type Error = index_checker::InvalidIndexError;

    fn into_type(self, contents: &'a ModuleContents) -> Result<&'a type_system::Type, Self::Error> {
        match self {
            type_system::Reference::Inline(ty) => Ok(ty),
            type_system::Reference::Index(index) => index_checker::get_type(*index, contents),
        }
    }
}

impl<'a, T: IntoType<'a> + Copy> IntoType<'a> for &T {
    type Error = T::Error;

    fn into_type(self, contents: &'a ModuleContents) -> Result<&'a type_system::Type, Self::Error> {
        T::into_type(*self, contents)
    }
}

pub(crate) fn resolve_many<'a, 't, T, I>(
    types: I,
    buffer: &'a mut Vec<&'t type_system::Type>,
    contents: &'t ModuleContents,
) -> Result<&'a [&'t type_system::Type], T::Error>
where
    T: IntoType<'t>,
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
