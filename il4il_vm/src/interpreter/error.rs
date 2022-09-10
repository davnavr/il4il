//! Module for interacting with interpreter errors.

use std::fmt::{Debug, Formatter};

/// The list of errors that can occur during interpretation of IL4IL bytecode.
#[derive(Debug, thiserror::Error)]
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
    /// Used when an instruction that cannot be interpreted by this version of the [`Interpreter`] is encountered.
    ///
    /// [`Interpreter`]: crate::interpreter::Interpreter
    #[error("cannot interpret {0:?} instruction")]
    UnsupportedInstruction(il4il::instruction::Instruction),
    #[error("host function error: {0}")]
    HostFunctionError(#[source] Box<dyn std::error::Error + Send + Sync>),
}

struct ErrorInner {
    kind: ErrorKind,
    //stack_trace: ,
}

/// The error type used for interpreter errors.
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "error: {}", self.kind())
        // TODO: Write the stack trace.
    }
}

impl std::error::Error for Error {}
