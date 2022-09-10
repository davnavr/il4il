//! Module for manipulation of locations in the source code.

use std::cmp::Ordering;

/// Represents a line or column number.
pub type Number = std::num::NonZeroUsize;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Location {
    pub line: Number,
    pub column: Number,
}

impl Location {
    pub fn new(line: Number, column: Number) -> Self {
        Self { line, column }
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            c => c,
        }
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
