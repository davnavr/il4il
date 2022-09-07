//! Module for manipulating IL4IL module names.

use crate::identifier::Id;
use std::borrow::Cow;

/// Specifies the name of a module.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ModuleName<'data> {
    pub name: Cow<'data, Id>,
    // TODO: Include module version numbers, means that version conflicts might appear at compile/link time
}

impl<'data> ModuleName<'data> {
    pub fn from_name<N: Into<Cow<'data, Id>>>(name: N) -> Self {
        Self { name: name.into() }
    }
}
