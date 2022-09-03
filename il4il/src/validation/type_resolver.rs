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

impl IntoType for &type_system::Reference {
    type Error = index_checker::InvalidIndexError;

    fn into_type(self, contents: &ModuleContents) -> Result<type_system::Type, Self::Error> {
        match self {
            type_system::Reference::Inline(ty) => Ok(*ty),
            type_system::Reference::Index(index) => index_checker::get_type(*index, contents).copied(),
        }
    }
}
