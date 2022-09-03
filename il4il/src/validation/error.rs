//! Contains types decribing errors that occur during IL4IL module validation.

use crate::index;
use std::fmt::{Display, Formatter};

/// A list specifying the different ways in which an IL4IL instruction is considered invalid.
///
/// Used with the [`InvalidInstructionError`] type.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum InvalidInstructionKind {
    #[error("expected terminator instruction at end of block")]
    ExpectedTerminator,
    #[error("no instructions should come after the first terminator instruction")]
    ExpectedTerminatorAsLastInstruction,
}

/// Indicates the location of an invalid IL4IL instruction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidInstructionLocation {
    instruction: crate::instruction::Instruction,
    index: usize,
}

impl InvalidInstructionLocation {
    pub(crate) fn new(instruction: crate::instruction::Instruction, index: usize) -> Self {
        Self { instruction, index }
    }
}

impl Display for InvalidInstructionLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "instruction \"{:?}\" at index {}", &self.instruction, self.index)
    }
}

/// The error type used when an IL4IL instruction is invalid.
///
/// Used with the [`ErrorKind`] type to indicate that an IL4IL basic block is not valid.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub struct InvalidInstructionError {
    block_index: index::Block,
    location: Option<InvalidInstructionLocation>,
    kind: InvalidInstructionKind,
}

impl InvalidInstructionError {
    pub fn new<K: Into<InvalidInstructionKind>>(block_index: index::Block, location: Option<InvalidInstructionLocation>, kind: K) -> Self {
        Self {
            block_index,
            location,
            kind: kind.into(),
        }
    }
}

impl Display for InvalidInstructionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid ")?;

        if let Some(location) = self.location.as_ref() {
            write!(f, "{location} in ")?;
        }

        write!(f, "code block {}: {}", self.block_index, &self.kind)
    }
}

/// A list specifying the kinds of errors that can occur during IL4IL module validation.
///
/// Usually used with the [`Error`] type.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error(transparent)]
    DuplicateSymbol(#[from] crate::symbol::DuplicateSymbolError),
    #[error(transparent)]
    InvalidInstruction(#[from] InvalidInstructionError),
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
