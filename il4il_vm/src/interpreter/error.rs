//! Module for interacting with interpreter errors.

use std::fmt::{Debug, Formatter};

/// The list of errors that can occur during interpretation of IL4IL bytecode.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Used when the [`Interpreter`] no longer has any code left to execute.
    ///
    /// [`Interpreter`]: crate::interpreter::Interpreter
    #[error("no code left to execute")]
    EndOfProgram,
    /// Used when an [`Unreachable`] terminator instruction is encountered.
    ///
    /// [`Unreachable`]: il4il::instruction::Instruction::Unreachable
    #[error("encountered unreachable point in code")]
    EncounteredUnreachable,
    #[error("cannot interpret {0:?} instruction")]
    UnsupportedInstruction(il4il::instruction::Instruction),
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
    pub(super) fn new(kind: ErrorKind) -> Self {
        Self(Box::new(ErrorInner { kind }))
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error").field("kind", self.kind()).finish()
    }
}
