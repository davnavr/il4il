//! Contains types that model the sections of an IL4IL module.

use crate::identifier::Id;
use std::borrow::Cow;

crate::kind_enum! {
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

crate::kind_enum! {
    /// Indicates the kind of section.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum SectionKind : u8 {
        Metadata = 0,
        Symbol = 3,
        Type = 4,
        FunctionSignature = 5,
        FunctionInstantiation = 6,
        FunctionDefinition = 8,
        Code = 9,
        EntryPoint = 10,
    }
}

/// Represents an IL4IL module section.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Section<'data> {
    /// The metadata section contains information about the module.
    Metadata(Vec<Metadata<'data>>),
    Symbol(Vec<crate::symbol::Assignment<'data>>),
    /// The type section stores commonly used types throughout the module.
    ///
    /// See also [`index::Type`].
    ///
    /// [`index::Type`]: crate::index::Type
    Type(Vec<crate::type_system::Type>),
    /// The function signature section stores the parameter and return types of functions throughout the module.
    ///
    /// See also [`index::FunctionSignature`].
    ///
    /// [`index::FunctionSignature`]: crate::index::FunctionSignature
    FunctionSignature(Vec<crate::function::Signature>),
    FunctionDefinition(Vec<crate::function::Definition>),
    FunctionInstantiation(Vec<crate::function::Instantiation>),
    /// The code section contains function bodies, which consist of basic blocks containing sequences of instructions.
    Code(Vec<crate::function::Body>),
    /// Specifies an entry point function for the module.
    EntryPoint(crate::index::FunctionInstantiation),
}

impl<'data> Section<'data> {
    #[must_use]
    pub fn into_owned<'owned>(self) -> Section<'owned> {
        match self {
            Self::Metadata(metadata) => Section::Metadata(metadata.into_iter().map(Metadata::into_owned).collect()),
            Self::Symbol(symbols) => Section::Symbol(symbols.into_iter().map(crate::symbol::Assignment::into_owned).collect()),
            Self::Type(types) => Section::Type(types),
            Self::FunctionSignature(signatures) => Section::FunctionSignature(signatures),
            Self::FunctionInstantiation(instantiations) => Section::FunctionInstantiation(instantiations),
            Self::FunctionDefinition(definitions) => Section::FunctionDefinition(definitions),
            Self::Code(code) => Section::Code(code),
            Self::EntryPoint(index) => Section::EntryPoint(index),
        }
    }

    #[must_use]
    pub fn kind(&self) -> SectionKind {
        match self {
            Self::Metadata(_) => SectionKind::Metadata,
            Self::Symbol(_) => SectionKind::Symbol,
            Self::Type(_) => SectionKind::Type,
            Self::FunctionSignature(_) => SectionKind::FunctionSignature,
            Self::FunctionInstantiation(_) => SectionKind::FunctionInstantiation,
            Self::FunctionDefinition(_) => SectionKind::FunctionDefinition,
            Self::Code(_) => SectionKind::Code,
            Self::EntryPoint(_) => SectionKind::EntryPoint,
        }
    }
}
