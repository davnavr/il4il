//! Parser error handling.

/// Error type used when parsing fails.
///
/// Used with [`Report`] to describe a parser error.
#[derive(Debug, thiserror::Error)]
#[error("parser error occured at file offset {file_offset:#X}")]
pub struct Error {
    file_offset: usize,
}

impl Error {
    pub(super) fn new(file_offset: usize) -> Self {
        Self { file_offset }
    }

    /// A file offset to the location where an error occured.
    pub fn file_offset(&self) -> usize {
        self.file_offset
    }
}

/// Describes a parser error.
pub type Report = error_stack::Report<Error>;
