//! The IL4IL assembly lexer.

use crate::cache::StringCache;
use crate::input::{self, Input};
use crate::location;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
/// The type of tokens used by the IL4IL assembler.
pub enum Token<'src> {
    OpenBracket,
    CloseBracket,
    Semicolon,
    Directive(&'src str),
    Word(&'src str),
    Unknown(&'src str),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        match self {
            Self::OpenBracket => f.write_char('{'),
            Self::CloseBracket => f.write_char('}'),
            Self::Semicolon => f.write_char(';'),
            Self::Directive(name) => {
                f.write_char('.')?;
                f.write_str(name)
            }
            Self::Word(word) => f.write_str(word),
            Self::Unknown(contents) => f.write_str(contents),
        }
    }
}

/// Represents a line of source code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Line {
    bytes: Range<usize>,
    line: location::Number,
}

impl Line {
    fn new(bytes: Range<usize>, line: location::Number) -> Self {
        Self { bytes, line }
    }

    /// The byte offsets marking the start and end of this line.
    ///
    /// The [`start`] offset is either `0` referring to the start of the file, or refers to the first non-newline character in the line.
    ///
    /// [`start`]: std::ops::Range::start
    pub fn byte_offsets(&self) -> &Range<usize> {
        &self.bytes
    }

    pub fn line_number(&self) -> location::Number {
        self.line
    }
}

pub struct OffsetsBuilder {
    lines: Vec<Line>,
    previous_offset: usize,
    line_number: location::Number,
}

impl OffsetsBuilder {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            previous_offset: 0,
            line_number: location::Number::START,
        }
    }

    fn new_line(&mut self, current_offset: usize) {
        debug_assert!(current_offset >= self.previous_offset);
        self.lines.push(Line::new(self.previous_offset..current_offset, self.line_number));
        self.previous_offset = current_offset + 1;
        self.line_number.increment();
    }

    fn finish(mut self, byte_length: usize) -> Offsets {
        match self.lines.last() {
            Some(last) if last.bytes.end < byte_length - 1 => self.new_line(byte_length),
            _ => (),
        }

        Offsets {
            byte_length,
            lines: self.lines,
        }
    }
}

/// Maps byte offsets to line and column number pairs.
#[derive(Clone, Debug, Default)]
pub struct Offsets {
    byte_length: usize,
    lines: Vec<Line>,
}

impl Offsets {
    /// Returns the length, in bytes, of the original input string.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Returns a [`Range`] of byte offsets over the original input string.
    pub fn offsets(&self) -> Range<usize> {
        0..self.byte_length
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Gets a line and column number corresponding to a byte offset.
    pub fn get_location(&self, byte_offset: usize) -> location::Location {
        let line = match self.lines.binary_search_by_key(&byte_offset, |line| line.bytes.start) {
            Ok(index) => &self.lines[index],
            Err(index) => {
                if let Some(line) = self.lines.get(index - 1) {
                    line
                } else {
                    return location::Location::new(location::Number::START, location::Number::new(byte_offset + 1).unwrap());
                }
            }
        };

        location::Location::new(
            line.line_number(),
            location::Number::new(byte_offset - line.byte_offsets().start + 1).unwrap(),
        )
    }

    pub fn get_location_range(&self, byte_offsets: Range<usize>) -> Range<location::Location> {
        Range {
            start: self.get_location(byte_offsets.start),
            end: self.get_location(byte_offsets.end),
        }
    }

    pub fn last_location(&self) -> location::Location {
        self.get_location(self.byte_length)
    }

    pub fn locations(&self) -> impl std::iter::ExactSizeIterator<Item = location::Location> + '_ {
        self.offsets().into_iter().map(|offset| self.get_location(offset))
    }
}

#[derive(Clone, Debug)]
pub struct Output<'cache> {
    pub(crate) tokens: Vec<(Token<'cache>, Range<usize>)>,
    pub(crate) strings: &'cache StringCache<'cache>,
    pub(crate) offsets: Offsets,
}

impl<'cache> Output<'cache> {
    pub fn tokens(&self) -> &[(Token<'cache>, Range<usize>)] {
        &self.tokens
    }

    pub fn offsets(&self) -> &Offsets {
        &self.offsets
    }
}

/// Produces a sequence of tokens from a string.
///
/// # Examples
///
/// ```
/// use il4il_asm::lexer::{self, Token};
///
/// assert_eq!(
///     lexer::tokenize(".metadata {}").tokens(),
///     &[
///         (Token::Directive("metadata"), 0..9),
///         (Token::OpenBracket, 10..11),
///         (Token::CloseBracket, 11..12),
///     ]
/// );
/// ```
pub fn tokenize<'cache, I: input::IntoInput>(
    source: I,
    string_cache: &'cache StringCache<'cache>,
) -> Result<Output<'cache>, <I::Source as Input>::Error> {
    let mut input = source.into_input();
    let mut tokens = Vec::new();
    let mut offsets = OffsetsBuilder::new();
    let mut byte_length = 0;

    // while let Some(tok) = lexer.next() {
    //     let offset = lexer.span();

    //     let actual_token = match tok {
    //         Tok::OpenBracket => Token::OpenBracket,
    //         Tok::CloseBracket => Token::CloseBracket,
    //         Tok::Semicolon => Token::Semicolon,
    //         Tok::Directive(name) => Token::Directive(name),
    //         Tok::Word(word) => Token::Word(word),
    //         Tok::Newline => {
    //             lexer.extras.new_line(offset.start);
    //             continue;
    //         }
    //         Tok::Unknown => Token::Unknown(&source[offset.clone()]),
    //     };

    //     tokens.push((actual_token, offset));
    // }

    Ok(Output {
        tokens,
        strings: string_cache,
        offsets: offsets.finish(byte_length),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location;

    #[test]
    fn simple_directive_produces_correct_output() {
        let cache = StringCache::new();
        let output = tokenize("\n.section {\n}\n", &cache).unwrap();

        assert_eq!(
            output.tokens(),
            &[
                (Token::Directive("section"), 1..9),
                (Token::OpenBracket, 10..11),
                (Token::CloseBracket, 12..13),
            ]
        );

        assert_eq!(
            output.offsets().lines(),
            &[
                Line::new(0..0, location::Number::new(1).unwrap()),
                Line::new(1..11, location::Number::new(2).unwrap()),
                Line::new(12..13, location::Number::new(3).unwrap()),
            ]
        );

        assert_eq!(
            output.offsets().locations().collect::<Vec<_>>(),
            vec![
                location::Location::new(location::Number::START, location::Number::new(1).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(1).unwrap()), // 'm'
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(2).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(3).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(4).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(5).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(6).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(7).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(8).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(9).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(10).unwrap()),
                location::Location::new(location::Number::new(2).unwrap(), location::Number::new(11).unwrap()), // '\n'
                location::Location::new(location::Number::new(3).unwrap(), location::Number::new(1).unwrap()),  // '}'
                location::Location::new(location::Number::new(3).unwrap(), location::Number::new(2).unwrap()),  // '\n'
            ]
        )
    }

    #[test]
    fn lines_are_correct_for_input_with_newline_only() {
        let cache = StringCache::new();
        assert_eq!(
            tokenize("\n", &cache).unwrap().offsets().lines(),
            &[Line::new(0..0, location::Number::new(1).unwrap()),]
        );
    }

    #[test]
    fn lines_are_correct_when_no_trailing_newline() {
        let cache = StringCache::new();
        assert_eq!(
            tokenize("\n\n.section", &cache).unwrap().offsets().lines(),
            &[
                Line::new(0..0, location::Number::new(1).unwrap()),
                Line::new(1..1, location::Number::new(2).unwrap()),
                Line::new(2..10, location::Number::new(3).unwrap()),
            ]
        );
    }
}
