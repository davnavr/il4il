//! Manipulation of IL4IL module symbols.
//!
//! Symbols are unique names assigned to the contents of an IL4IL module, such as its functions.

#![deny(unsafe_code)]

use crate::identifier::{Id, Identifier};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

/// Indicates whether the symbol is visible outside of the containing module.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SymbolKind {
    Private,
    Export,
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
    pub fn new<N: Into<Cow<'data, Id>>>(kind: SymbolKind, name: N) -> Self {
        Self(match name.into() {
            Cow::Borrowed(borrowed) => match kind {
                SymbolKind::Export => Contents::ExportBorrowed(borrowed),
                SymbolKind::Private => Contents::PrivateBorrowed(borrowed),
            },
            Cow::Owned(owned) => match kind {
                SymbolKind::Export => Contents::ExportOwned(owned),
                SymbolKind::Private => Contents::PrivateOwned(owned),
            },
        })
    }

    pub fn kind(&self) -> SymbolKind {
        match self.0 {
            Contents::ExportBorrowed(_) | Contents::ExportOwned(_) => SymbolKind::Export,
            Contents::PrivateBorrowed(_) | Contents::PrivateOwned(_) => SymbolKind::Private,
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
