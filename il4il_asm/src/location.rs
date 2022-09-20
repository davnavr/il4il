//! Module for manipulation of locations in the source code.

use std::cmp::Ordering;
use std::num::NonZeroUsize;

/// Represents a line or column number.
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Number(NonZeroUsize);

impl Number {
    pub const START: Self = Self(unsafe {
        // Safety: 1 != 0
        NonZeroUsize::new_unchecked(1)
    });

    pub fn new(number: usize) -> Option<Self> {
        NonZeroUsize::new(number).map(Self)
    }

    pub(crate) fn increment(&mut self) {
        self.0 = NonZeroUsize::new(self.0.get() + 1).unwrap();
    }
}

impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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
