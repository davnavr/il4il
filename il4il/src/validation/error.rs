//! Contains types decribing errors that occur during IL4IL module validation.

use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub struct InvalidIndexError {
    kind: &'static str,
    index: usize,
    maximum: Option<usize>,
}

impl InvalidIndexError {
    pub(crate) fn new<S: crate::index::IndexSpace>(index: crate::index::Index<S>, maximum: Option<usize>) -> Self {
        Self {
            kind: S::name(),
            index: usize::from(index),
            maximum,
        }
    }
}

impl Display for InvalidIndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} index #{} is out of bounds", self.kind, self.index)?;
        if let Some(maximum) = self.maximum {
            write!(f, "maximum valid index is #{}", maximum)?;
        }
        Ok(())
    }
}

/// A list specifying the kinds of errors that can occur during IL4IL module validation.
///
/// Usually used with the [`Error`] type.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error(transparent)]
    IndexOutOfBounds(#[from] InvalidIndexError),
    #[error(transparent)]
    DUplicateSymbol(#[from] crate::symbol::DuplicateSymbolError),
}

/// Represents an error that occured during the validation of an IL4IL module.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[error(transparent)]
#[repr(transparent)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn from_kind<E: Into<ErrorKind>>(kind: E) -> Self {
        Self(Box::new(kind.into()))
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(error: E) -> Self {
        Self::from_kind(error)
    }
}
