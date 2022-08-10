//! Module for manipulating IL4IL identifier strings.
//!
//! For more information, see the documentation for [`Id`].
//!
//! [`Id`] is to [`Identifier`] as [`str`] is to [`String`].

use std::borrow::{Borrow, ToOwned};
use std::convert::AsRef;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

/// The error type used to indicate that a string is not a valid IL4IL identifier.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum InvalidError {
    #[error("identifiers cannot be empty")]
    Empty,
    #[error("identifiers cannot contain null bytes")]
    ContainsNull,
}

/// The error type used when parsing a IL4IL identifier from a sequence of bytes fails.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    InvalidIdentifier(#[from] InvalidError),
    #[error(transparent)]
    InvalidSequence(#[from] std::str::Utf8Error),
}

/// Represents a IL4IL identifier string, which is a valid UTF-8 string that cannot be empty or contain any `NUL` bytes.
///
/// The requirements placed on identifiers ensures conversions to other formats are easier. For example, LLVM uses null terminated
/// strings which IL4IL strings would be compatible with.
///
/// Additionally, [`Id`] does not provide methods to mutate or manipulate identifier strings, in order to ensure that its
/// invariants hold.
#[derive(Eq, Hash, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Id(str);

impl Id {
    /// Returns the contents of the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Copies the contents of the identifier string into a heap allocation.
    #[must_use]
    pub fn to_identifier(&self) -> Identifier {
        Identifier(String::from(self.as_str()))
    }

    /// Creates a reference to an identfier from a string, without any validation checks.
    ///
    /// # Safety
    ///
    /// Callers should ensure that the string does not contain any interior `NUL` bytes and must not be empty.
    #[must_use]
    pub unsafe fn from_str_unchecked(identifier: &str) -> &Id {
        unsafe {
            // Safety: Representation of Id allows a safe transmutation
            std::mem::transmute::<&str, &Id>(identifier)
        }
    }

    /// Attempts to create a reference to an identifier string.
    ///
    /// If an owned [`Identifier`] is needed, use [`Identifier::from_string`] or [`std::str::FromStr`] instead.
    ///
    /// # Errors
    ///
    /// If the string is empty or contains a `NUL` character, then an error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::identifier::*;
    /// assert_eq!(Id::new("very_very_long_function_name").map(Id::as_str), Ok("very_very_long_function_name"));
    /// assert_eq!(Id::new(""), Err(InvalidError::Empty));
    /// assert_eq!(Id::new("\0"), Err(InvalidError::ContainsNull));
    /// ```
    pub fn new(identifier: &str) -> Result<&Self, InvalidError> {
        if identifier.is_empty() {
            Err(InvalidError::Empty)
        } else if identifier.bytes().any(|b| b == 0) {
            Err(InvalidError::ContainsNull)
        } else {
            // Safety: Validation is performed above
            Ok(unsafe { Self::from_str_unchecked(identifier) })
        }
    }

    /// Converts a slice of bytes into a IL4IL identifier string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::identifier::*;
    /// assert!(Id::from_utf8(&[]).is_err());
    /// assert!(Id::from_utf8(&[0u8]).is_err());
    /// ```
    pub fn from_utf8(bytes: &[u8]) -> Result<&Id, ParseError> {
        Ok(Self::new(std::str::from_utf8(bytes)?)?)
    }

    /// Converts a boxed identifier into a boxed string.
    #[must_use]
    pub fn into_boxed_str(self: Box<Id>) -> Box<str> {
        unsafe {
            // Safety: Layout of str and id is identical
            std::mem::transmute(self)
        }
    }

    /// Turns a boxed identifier string into an [`Identifier`].
    #[must_use]
    pub fn into_identifier(self: Box<Id>) -> Identifier {
        Identifier(self.into_boxed_str().into())
    }
}

impl Deref for Id {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        Id::as_str(self)
    }
}

impl AsRef<std::path::Path> for Id {
    fn as_ref(&self) -> &std::path::Path {
        self.as_str().as_ref()
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl ToOwned for Id {
    type Owned = Identifier;

    fn to_owned(&self) -> Self::Owned {
        self.to_identifier()
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

/// Owned form of a IL4IL identifier string.
///
/// For more information, see the documentation for [`Id`].
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Identifier(String);

impl Identifier {
    /// Returns the contents of this identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a reference to the underlying [`String`].
    #[must_use]
    pub fn as_string(&self) -> &String {
        &self.0
    }

    /// Returns a borrowed version of this identifier string.
    #[must_use]
    pub fn as_id(&self) -> &Id {
        unsafe {
            // Safety: String is assumed to be a valid identifier
            Id::from_str_unchecked(&self.0)
        }
    }

    /// Creates an owned version of an identifier string.
    #[must_use]
    pub fn from_id(identifier: &Id) -> Self {
        identifier.to_identifier()
    }

    /// Converts a boxed identifier string into an [`Identifier`].
    #[must_use]
    pub fn from_boxed_id(identifier: Box<Id>) -> Self {
        Self(identifier.into_boxed_str().into())
    }

    /// Attempts to convert a [`String`] into an identifier.
    ///
    /// # Errors
    ///
    /// If the string is empty or contains a `NUL` character, then an error is returned.
    pub fn from_string(identifier: String) -> Result<Self, InvalidError> {
        Id::new(&identifier)?;
        Ok(Self(identifier))
    }

    /// Creates an owned identifier string without any validation checks.
    ///
    /// # Safety
    ///
    /// See [`Id::from_str_unchecked`] for more information.
    pub unsafe fn from_string_unchecked(identifier: String) -> Self {
        Self(identifier)
    }

    /// Converts a boxed string into an identifier.
    ///
    /// # Errors
    ///
    /// If the string is empty or contains a `NUL` character, then an error is returned.
    pub fn from_boxed_str(identifier: Box<str>) -> Result<Self, InvalidError> {
        Self::from_string(identifier.into())
    }

    /// Appends an identifier string to the end of this identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::identifier::*;
    /// # use std::str::FromStr;
    /// let mut id = Identifier::from_str("MyName").unwrap();
    /// id.push_id(Id::new("IsValid").unwrap());
    /// assert_eq!(id.as_str(), "MyNameIsValid");
    /// ```
    pub fn push_id(&mut self, identifier: &Id) {
        self.0.push_str(identifier.as_str());
    }

    /// Returns the underlying [`String`].
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for Identifier {
    type Target = Id;

    fn deref(&self) -> &Id {
        self.as_id()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<String> for Identifier {
    fn as_ref(&self) -> &String {
        self.as_string()
    }
}

impl AsRef<std::path::Path> for Identifier {
    fn as_ref(&self) -> &std::path::Path {
        self.as_str().as_ref()
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<String> for Identifier {
    fn borrow(&self) -> &String {
        self.as_string()
    }
}

impl Borrow<Id> for Identifier {
    fn borrow(&self) -> &Id {
        self.as_id()
    }
}

impl std::str::FromStr for Identifier {
    type Err = InvalidError;

    fn from_str(identifier: &str) -> Result<Self, Self::Err> {
        Id::new(identifier).map(Id::to_identifier)
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(self.as_id(), f)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(self.as_id(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propcheck;

    impl propcheck::Arb for Identifier {
        type Shrinker = std::iter::Empty<Self>;

        fn arbitrary<R: propcheck::Rng + ?Sized>(gen: &mut propcheck::Gen<'_, R>) -> Self {
            loop {
                if let Ok(identifier) = Self::from_string(String::arbitrary(gen)) {
                    return identifier;
                }
            }
        }

        fn shrink(&self) -> Self::Shrinker {
            std::iter::empty()
        }
    }

    propcheck::property! {
        fn all_identifiers_are_valid(identifier: Identifier) {
            propcheck::assertion!(Id::new(identifier.as_str()).is_ok())
        }
    }

    propcheck::property! {
        fn two_appended_identifiers_are_valid(first: Identifier, second: Identifier) {
            let mut identifier = first;
            identifier.push_id(second.as_id());
            propcheck::assertion!(Id::new(identifier.as_str()).is_ok())
        }
    }
}
