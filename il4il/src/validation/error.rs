//! Contains types decribing errors that occur during IL4IL module validation.

/// A list specifying the kinds of errors that can occur during IL4IL module validation.
///
/// Usually used with the [`Error`] type.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {}

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
