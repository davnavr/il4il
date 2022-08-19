//! Manipulation of indices used to refer to the different contents of a module.

use crate::integer::{self, VarU28};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

mod private {
    pub trait Sealed {}
}

/// Specifies the names of various kinds of indices.
///
/// This trait is sealed and is not meant to be implemented outside of this crate.
pub trait IndexSpace: private::Sealed {
    fn name() -> &'static str;
}

macro_rules! index_space {
    ($(#[$meta:meta])* $vis:vis struct $name:ident {
        const NAME = $s:literal;
    }) => {
        $(#[$meta])*
        #[derive(Debug, Eq, PartialEq)]
        #[non_exhaustive]
        $vis struct $name;

        impl IndexSpace for $name {
            fn name() -> &'static str {
                $s
            }
        }

        impl private::Sealed for $name {}
    };
}

/// An index to some content within an IL4IL module.
#[repr(transparent)]
#[non_exhaustive]
pub struct Index<S: IndexSpace> {
    pub index: usize,
    _phantom: PhantomData<S>,
}

impl<S: IndexSpace> Index<S> {
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}

impl<S: IndexSpace> From<usize> for Index<S> {
    fn from(index: usize) -> Self {
        Self::new(index)
    }
}

impl<S: IndexSpace> From<Index<S>> for usize {
    fn from(index: Index<S>) -> usize {
        index.index
    }
}

impl<S: IndexSpace> TryFrom<integer::VarU28> for Index<S> {
    type Error = std::num::TryFromIntError;

    fn try_from(value: integer::VarU28) -> Result<Self, Self::Error> {
        usize::try_from(value).map(Self::new)
    }
}

impl<S: IndexSpace> TryFrom<Index<S>> for VarU28 {
    type Error = integer::EncodingError;

    fn try_from(index: Index<S>) -> Result<Self, Self::Error> {
        Self::try_from(index.index)
    }
}

impl<S: IndexSpace> Clone for Index<S> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _phantom: PhantomData,
        }
    }
}

impl<S: IndexSpace> Copy for Index<S> {}

impl<S: IndexSpace> Debug for Index<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Index").field(&self.index).finish()
    }
}

impl<S: IndexSpace> Display for Index<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} #{}", S::name(), self.index)
    }
}

impl<S: IndexSpace> PartialEq for Index<S> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<S: IndexSpace> Eq for Index<S> {}

impl<S: IndexSpace> Ord for Index<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<S: IndexSpace> PartialOrd for Index<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

index_space! {
    pub struct TypeSpace {
        const NAME = "type";
    }
}

/// Type indices refer to the contents of all type sections within a module, with `0` referring to the first type of the first type
/// section.
pub type Type = Index<TypeSpace>;
