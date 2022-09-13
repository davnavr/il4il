//! Types representing literals in the source.

use std::fmt::{Display, Formatter};

/// Indicates the base of an [`Integer`] literal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum IntegerBase {
    Binary,
    Decimal,
    Hexadecimal,
}

impl Display for IntegerBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decimal => Ok(()),
            Self::Binary => f.write_str("0b"),
            Self::Hexadecimal => f.write_str("0x"),
        }
    }
}

/// Represents a literal integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Integer<'src> {
    digits: &'src str,
    base: IntegerBase,
}

impl<'src> Integer<'src> {
    pub(crate) fn new(base: IntegerBase, digits: &'src str) -> Self {
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

    pub fn base(&self) -> IntegerBase {
        self.base
    }
}

impl Display for Integer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.base, self.digits)
    }
}
