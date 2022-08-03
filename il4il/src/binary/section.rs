//! Module to manipulate IL4IL module sections.

use crate::identifier::Id;
use std::borrow::Cow;

macro_rules! kind_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$case_meta:meta])* $case_name:ident = $case_number:literal,)*
    }) => {
        $(#[$_meta])*
        $vis enum $name {
            $($(#[$case_meta])* $case_name = $case_number,)*
        }
    };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MetadataKind {
    Name,
}

impl From<MetadataKind> for u8 {
    fn from(kind: MetadataKind) -> Self {
        match kind {
            MetadataKind::Name => 0,
        }
    }
}

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

    #[must_use]
    pub fn kind(&self) -> MetadataKind {
        match self {
            Self::Name(_) => MetadataKind::Name,
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
