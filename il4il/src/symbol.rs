//! Manipulation of IL4IL module symbols.
//!
//! Symbols are unique names assigned to the contents of an IL4IL module, such as its functions.

#![deny(unsafe_code)]

use crate::identifier::{Id, Identifier};
use crate::index;
use std::borrow::{Borrow, Cow};
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

/// Assigns symbol names to indices to content within a module.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Assignment<'data> {
    pub symbols: Vec<(Cow<'data, Id>, usize)>,
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

/// Represents an index to content within a module that is capable of having a symbol.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetIndex {
    FunctionTemplate(index::FunctionTemplate),
}

impl Display for TargetIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FunctionTemplate(index) => Display::fmt(&index, f),
        }
    }
}

impl TargetKind {
    pub fn create_index(self, index: usize) -> TargetIndex {
        match self {
            Self::FunctionTemplate => TargetIndex::FunctionTemplate(index.into()),
        }
    }
}

/// Indicates why a symbol could not be added to a [`Lookup`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DuplicateSymbolKind {
    #[error("symbol is already assigned to {0}")]
    ExistingIndex(TargetIndex),
    #[error("index already assigned the symbol {0:?}")]
    ExistingSymbol(Identifier),
}

#[derive(Clone, Eq, PartialEq)]
struct DuplicateSymbolErrorInner {
    symbol: Identifier,
    index: TargetIndex,
    kind: DuplicateSymbolKind,
}

/// Error type used when an attempt to add a symbol to a [`Lookup`] would result in a symbol corresponding to more than one index, or an
/// index being associated with more than one symbol.
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DuplicateSymbolError(Box<DuplicateSymbolErrorInner>);

impl DuplicateSymbolError {
    fn new(symbol: Identifier, index: TargetIndex, kind: DuplicateSymbolKind) -> Self {
        Self(Box::new(DuplicateSymbolErrorInner { symbol, index, kind }))
    }

    pub fn kind(&self) -> &DuplicateSymbolKind {
        &self.0.kind
    }
}

impl Debug for DuplicateSymbolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuplicateSymbolError")
            .field("symbol", &self.0.symbol)
            .field("index", &self.0.index)
            .field("kind", &self.0.kind)
            .finish()
    }
}

impl Display for DuplicateSymbolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attempt to assign index {} the symbol {:?}, but {}",
            self.0.index, &self.0.symbol, &self.0.kind
        )
    }
}

impl std::error::Error for DuplicateSymbolError {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LookupEntry<'data> {
    index: TargetIndex,
    name: Cow<'data, Id>,
    kind: Kind,
}

impl LookupEntry<'_> {
    pub fn index(&self) -> TargetIndex {
        self.index
    }

    pub fn name(&self) -> &Id {
        &self.name
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Lookup<'data> {
    entries: Vec<LookupEntry<'data>>,
    index_lookup: rustc_hash::FxHashMap<Cow<'data, Id>, usize>,
    name_lookup: rustc_hash::FxHashMap<TargetIndex, usize>,
}

impl<'data> Lookup<'data> {
    pub fn from_assignments<'a, A>(assignments: A) -> Result<Self, DuplicateSymbolError>
    where
        A: IntoIterator<Item = &'a Assignment<'data>>,
        'data: 'a,
    {
        use std::collections::hash_map;

        let iterator = assignments.into_iter();
        let mut lookup = {
            let capacity = {
                let (minimum_count, maximum_count) = iterator.size_hint();
                maximum_count.unwrap_or(minimum_count)
            };

            Self {
                entries: Vec::with_capacity(capacity),
                index_lookup: rustc_hash::FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
                name_lookup: rustc_hash::FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            }
        };

        for assignment in iterator {
            let symbol_kind = assignment.symbol_kind();
            let create_target_index = {
                let target_kind = assignment.target_kind();
                move |index| target_kind.create_index(index)
            };

            for (name, index) in assignment.symbols.iter() {
                let entry_index = lookup.entries.len();
                let target_index = create_target_index(*index);

                match lookup.index_lookup.entry(name.clone()) {
                    hash_map::Entry::Vacant(vacant) => {
                        vacant.insert(entry_index);
                    }
                    hash_map::Entry::Occupied(occupied) => {
                        return Err(DuplicateSymbolError::new(
                            name.clone().into_owned(),
                            target_index,
                            DuplicateSymbolKind::ExistingIndex(lookup.entries[*occupied.get()].index),
                        ))
                    }
                }

                match lookup.name_lookup.entry(target_index) {
                    hash_map::Entry::Vacant(vacant) => {
                        vacant.insert(entry_index);
                    }
                    hash_map::Entry::Occupied(occupied) => {
                        return Err(DuplicateSymbolError::new(
                            name.clone().into_owned(),
                            target_index,
                            DuplicateSymbolKind::ExistingSymbol(lookup.entries[*occupied.get()].name.clone().into_owned()),
                        ))
                    }
                }

                lookup.entries.push(LookupEntry {
                    index: target_index,
                    name: name.clone(),
                    kind: symbol_kind,
                });
            }
        }

        Ok(lookup)
    }

    /// Gets the index to module data corresponding to a particular symbol.
    pub fn get_index<S>(&self, symbol: &S) -> Option<&LookupEntry>
    where
        S: ?Sized,
        Cow<'data, Id>: Borrow<S>,
        S: std::hash::Hash + Eq,
    {
        self.index_lookup.get(symbol).copied().map(|index| &self.entries[index])
    }

    /// Gets the symbol corresponding to the specified index, or `None` if a symbol was not defined for this index.
    pub fn get_symbol<I: Into<TargetIndex>>(&self, index: I) -> Option<&LookupEntry> {
        self.name_lookup.get(&index.into()).copied().map(|index| &self.entries[index])
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = &LookupEntry> {
        self.entries.iter()
    }
}

impl Debug for Lookup<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.entries.iter()).finish()
    }
}
