//! Module to manipulate IL4IL module sections.

use crate::identifier::Id;
use std::borrow::Cow;

/// Describes an IL4IL module.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Metadata<'data> {
    /// Specifies the name of an IL4IL module.
    Name(Cow<'data, Id>),
}

impl<'data> Metadata<'data> {
    #[must_use]
    pub fn into_owned<'owned>(self) -> Metadata<'owned> {
        match self {
            Self::Name(name) => Metadata::Name(Cow::Owned(name.into_owned())),
        }
    }
}

/// Represents an IL4IL module section.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Section<'data> {
    /// The metadata section contains information about the module.
    Metadata(Vec<Metadata<'data>>),
}

impl<'data> Section<'data> {
    #[must_use]
    pub fn into_owned<'owned>(self) -> Section<'owned> {
        match self {
            Self::Metadata(metadata) => Section::Metadata(metadata.into_iter().map(Metadata::into_owned).collect()),
        }
    }
}
