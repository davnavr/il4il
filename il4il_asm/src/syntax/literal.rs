//! Types representing literals in the source.

use std::fmt::{Display, Formatter};

/// Represents a literal integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Integer<'src> {
    digits: &'src str,
    base: Option<char>,
}

impl<'src> Integer<'src> {
    pub fn new(base: Option<char>, digits: &'src str) -> Self {
        Self { base, digits }
    }

    /// Returns the digits as they were in the source text, including any digit separators ('_').
    pub fn original_digits(&self) -> &'src str {
        self.digits
    }

    /// Returns the digits of the integer literal, omitting any digit separators ('_').
    pub fn iter_digits(&self) -> impl std::iter::FusedIterator<Item = char> + 'src {
        self.digits.chars().filter(|c| *c != '_')
    }

    pub fn base(&self) -> Option<char> {
        self.base
    }
}

impl Display for Integer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        if let Some(c) = self.base {
            f.write_char('0')?;
            f.write_char(c)?;
        }
        f.write_str(self.digits)
    }
}
