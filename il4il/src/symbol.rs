//! Manipulation of IL4IL module symbols.
//!
//! Symbols are unique names assigned to the contents of an IL4IL module, such as its functions.

#![deny(unsafe_code)]

use crate::identifier::{Id, Identifier};
use crate::index;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

crate::kind_enum! {
    /// Indicates whether the symbol is accessible outside of the containing module.
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum Kind : u8 {
        Private = 0,
        Export = 1,
    }
}

crate::kind_enum! {
    /// Represents the set of all things that can be assigned a symbol within a module.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub enum TargetKind : u8 {
        FunctionTemplate = 1,
    }
}

/// Represents an index to content within a module that is capable of having a symbol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetIndex {
    FunctionTemplate(index::FunctionTemplate),
}

/// Assigns content within a module corresponding to indices with symbol names.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Assignment<'data> {
    pub symbols: Vec<(usize, Cow<'data, Id>)>,
    symbol_kind: Kind,
    target_kind: TargetKind,
}

impl<'data> Assignment<'data> {
    pub fn new(symbol_kind: Kind, target_kind: TargetKind) -> Self {
        Self {
            symbols: Vec::new(),
            symbol_kind,
            target_kind,
        }
    }

    pub fn symbol_kind(&self) -> Kind {
        self.symbol_kind
    }

    pub fn target_kind(&self) -> TargetKind {
        self.target_kind
    }

    pub(crate) fn into_owned(self) -> Assignment<'static> {
        Assignment {
            symbols: self
                .symbols
                .into_iter()
                .map(|(index, name)| (index, Cow::Owned(name.into_owned())))
                .collect(),
            symbol_kind: self.symbol_kind,
            target_kind: self.target_kind,
        }
    }
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Contents<'data> {
    PrivateBorrowed(&'data Id),
    PrivateOwned(Identifier),
    ExportBorrowed(&'data Id),
    ExportOwned(Identifier),
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Symbol<'data>(Contents<'data>);

impl<'data> Symbol<'data> {
    pub fn new<N: Into<Cow<'data, Id>>>(kind: Kind, name: N) -> Self {
        Self(match name.into() {
            Cow::Borrowed(borrowed) => match kind {
                Kind::Export => Contents::ExportBorrowed(borrowed),
                Kind::Private => Contents::PrivateBorrowed(borrowed),
            },
            Cow::Owned(owned) => match kind {
                Kind::Export => Contents::ExportOwned(owned),
                Kind::Private => Contents::PrivateOwned(owned),
            },
        })
    }

    pub fn kind(&self) -> Kind {
        match self.0 {
            Contents::ExportBorrowed(_) | Contents::ExportOwned(_) => Kind::Export,
            Contents::PrivateBorrowed(_) | Contents::PrivateOwned(_) => Kind::Private,
        }
    }

    pub fn name(&self) -> &Id {
        match &self.0 {
            Contents::PrivateBorrowed(name) | Contents::ExportBorrowed(name) => name,
            Contents::PrivateOwned(name) | Contents::ExportOwned(name) => name.as_id(),
        }
    }

    pub fn into_name(self) -> Cow<'data, Id> {
        match self.0 {
            Contents::PrivateBorrowed(name) | Contents::ExportBorrowed(name) => Cow::Borrowed(name),
            Contents::PrivateOwned(name) | Contents::ExportOwned(name) => Cow::Owned(name),
        }
    }
}

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.name(), f)
    }
}
