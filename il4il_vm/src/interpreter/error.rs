//! Module for interacting with interpreter errors.

use std::fmt::{Debug, Formatter};

/// The list of errors that can occur during interpretation of IL4IL bytecode.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Used when an [`Unreachable`] terminator instruction is encountered.
    ///
    /// [`Unreachable`]: il4il::instruction::Instruction::Unreachable
    #[error("encountered unreachable point in code")]
    EncounteredUnreachable,
}

#[derive(Clone)]
struct ErrorInner {
    kind: ErrorKind,
}

/// The error type used for interpreter errors.
#[derive(Clone)]
#[repr(transparent)]
pub struct Error(Box<ErrorInner>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error").field("kind", self.kind()).finish()
    }
}
