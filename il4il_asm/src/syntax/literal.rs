//! Types representing literals in the source.

use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Deref;

/// Represents a literal integer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Integer<S: Deref<Target = str>> {
    digits: S,
    base: Option<char>,
}

impl<S: Deref<Target = str>> Integer<S> {
    pub fn new(base: Option<char>, digits: S) -> Self {
        Self { base, digits }
    }

    /// Returns the digits as they were in the source text, including any digit separators ('_').
    pub fn original_digits(&self) -> &S {
        &self.digits
    }

    /// Returns the digits of the integer literal, omitting any digit separators ('_').
    pub fn iter_digits(&self) -> impl std::iter::FusedIterator<Item = char> + '_ {
        self.digits.chars().filter(|c| *c != '_')
    }

    pub fn base(&self) -> Option<char> {
        self.base
    }
}

impl<S: Deref<Target = str>> Display for Integer<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.base {
            f.write_char('0')?;
            f.write_char(c)?;
        }
        f.write_str(&self.digits)
    }
}

/// Represents a literal string.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct String<S: Deref<Target = str>>(S);

impl<S: Deref<Target = str>> String<S> {
    pub fn new(contents: S) -> Self {
        Self(contents)
    }

    pub fn contents(&self) -> &S {
        &self.0
    }

    pub fn into_contents(self) -> S {
        self.0
    }
}

impl<S: Deref<Target = str>> Debug for String<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.deref(), f)
    }
}

impl<S: Deref<Target = str>> Display for String<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('\"')?;
        for c in self.0.chars() {
            match c {
                '\n' | '\t' | '\r' | '\\' | '\"' => {
                    f.write_char('\\')?;
                    f.write_char(c)?;
                }
                _ if c.is_ascii_graphic() || c == ' ' => f.write_char(c)?,
                _ => write!(f, "\\u{:#04X}", c as u32)?,
            }
        }
        f.write_char('\"')
    }
}

impl<S: Deref<Target = str>> std::ops::Deref for String<S> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
