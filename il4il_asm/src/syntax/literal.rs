//! Types representing literals in the source.

use std::fmt::{Debug, Display, Formatter};

/// Represents a literal integer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Integer<'a> {
    digits: &'a str,
    base: Option<char>,
}

impl<'a> Integer<'a> {
    pub fn new(base: Option<char>, digits: &'a str) -> Self {
        Self { base, digits }
    }

    /// Returns the digits as they were in the source text, including any digit separators ('_').
    pub fn original_digits(&self) -> &'a str {
        self.digits
    }

    /// Returns the digits of the integer literal, omitting any digit separators ('_').
    pub fn iter_digits(&self) -> impl std::iter::FusedIterator<Item = char> + 'a {
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

/// Represents a literal string.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct String<'a>(&'a str);

impl<'a> String<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents)
    }

    pub fn contents(self) -> &'a str {
        self.0
    }
}

impl Debug for String<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0, f)
    }
}

impl Display for String<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl std::ops::Deref for String<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
