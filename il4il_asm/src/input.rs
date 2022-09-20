//! Module for manipulating assembler input.
//!
//! See the documentation for the [`Input`] trait for more information.

use std::convert::Infallible;

/// Represents a sequence of characters.
pub trait Input {
    type Error;

    fn next(&mut self) -> Result<Option<char>, Self::Error>;
}

/// Provides an [`Input`] implementation over an [`Iterator`] of characters.
#[derive(Debug)]
#[repr(transparent)]
pub struct CharInput<I>(I);

impl<I: Iterator<Item = char>> From<I> for CharInput<I> {
    fn from(chars: I) -> Self {
        Self(chars)
    }
}

impl<I: Iterator<Item = char>> Input for CharInput<I> {
    type Error = Infallible;

    fn next(&mut self) -> Result<Option<char>, Self::Error> {
        Ok(self.0.next())
    }
}

/// Provides an [`Input`] implementation for an [`std::io::Read`] source.
///
/// # Examples
///
/// ```
/// use il4il_asm::input::{Input, ReadInput};
///
/// let bytes = [b'H', b'e', b'y'];
/// let mut input = ReadInput::from(bytes.as_slice());
/// assert_eq!(input.next().unwrap(), Some('H'));
/// assert_eq!(input.next().unwrap(), Some('e'));
/// assert_eq!(input.next().unwrap(), Some('y'));
/// assert_eq!(input.next().unwrap(), None);
/// ```
#[derive(Debug)]
#[repr(transparent)]
pub struct ReadInput<R>(R);

impl<R: std::io::Read> From<R> for ReadInput<R> {
    fn from(source: R) -> Self {
        Self(source)
    }
}

impl<R: std::io::Read> Input for ReadInput<R> {
    type Error = std::io::Error;

    fn next(&mut self) -> Result<Option<char>, Self::Error> {
        let mut buffer = [0u8; 4];

        if self.0.read(&mut buffer[0..1])? == 0 {
            return Ok(None);
        }

        let last_offset = match buffer[0].trailing_ones() {
            0 => 1,
            2 => 2,
            3 => 3,
            _ => 4,
        };

        std::str::from_utf8(&buffer[0..last_offset])
            .map(|s| s.chars().next())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Conversion into an [`Input`].
pub trait IntoInput {
    type Source: Input;

    fn into_input(self) -> Self::Source;
}

impl<I: Input> IntoInput for I {
    type Source = Self;

    fn into_input(self) -> Self::Source {
        self
    }
}

impl<'a> IntoInput for &'a str {
    type Source = CharInput<std::str::Chars<'a>>;

    fn into_input(self) -> Self::Source {
        CharInput::from(self.chars())
    }
}

impl<'a> IntoInput for &'a String {
    type Source = CharInput<std::str::Chars<'a>>;

    fn into_input(self) -> Self::Source {
        self.as_str().into_input()
    }
}

impl IntoInput for std::fs::File {
    type Source = ReadInput<Self>;

    fn into_input(self) -> Self::Source {
        ReadInput::from(self)
    }
}

impl<'a> IntoInput for &'a [u8] {
    type Source = ReadInput<&'a [u8]>;

    fn into_input(self) -> Self::Source {
        ReadInput::from(self)
    }
}
