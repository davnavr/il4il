//! Contains types that model the sections of an IL4IL module.

use crate::identifier::Id;
use std::borrow::Cow;

macro_rules! kind_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident : $inty:ty {
        $($(#[$case_meta:meta])* $case_name:ident = $case_number:literal,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$case_meta])* $case_name = $case_number,)*
        }

        impl $name {
            pub const fn new(value: $inty) -> Option<Self> {
                match value {
                    $(_ if value == $case_number => Some(Self::$case_name),)*
                    _ => None
                }
            }
        }

        impl From<$name> for $inty {
            fn from(kind: $name) -> Self {
                match kind {
                    $($name::$case_name => $case_number,)*
                }
            }
        }
    };
}

kind_enum! {
    /// Indicates the kind of metadata.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum MetadataKind : u8 {
        Name = 0,
    }
}

/// Describes an IL4IL module.
#[derive(Clone, Debug)]
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

kind_enum! {
    /// Indicates the kind of section.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum SectionKind : u8 {
        Metadata = 0,
        Type = 3,
    }
}

/// Represents an IL4IL module section.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Section<'data> {
    /// The metadata section contains information about the module.
    Metadata(Vec<Metadata<'data>>),
    /// The type section stores commonly used types throughout the module.
    Type(Vec<crate::type_system::Type>),
}

impl<'data> Section<'data> {
    #[must_use]
    pub fn into_owned<'owned>(self) -> Section<'owned> {
        match self {
            Self::Metadata(metadata) => Section::Metadata(metadata.into_iter().map(Metadata::into_owned).collect()),
            Self::Type(types) => Section::Type(types),
        }
    }

    #[must_use]
    pub fn kind(&self) -> SectionKind {
        match self {
            Self::Metadata(_) => SectionKind::Metadata,
            Self::Type(_) => SectionKind::Type,
        }
    }
}
