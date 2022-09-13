//! The IL4IL assembly lexer.

use crate::cache::StringCache;
use crate::input::{self, Input};
use crate::location;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
/// The type of tokens used by the IL4IL assembler.
pub enum Token<'cache> {
    OpenBracket,
    CloseBracket,
    Semicolon,
    Directive(&'cache str),
    Word(&'cache str),
    Unknown(&'cache str),
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

struct Characters<I: Input> {
    input: I,
    offset: usize,
    peeked_offset: usize,
    peeked: Option<Result<Option<char>, I::Error>>,
}

impl<I: Input> Characters<I> {
    fn new(input: I) -> Self {
        Self {
            input,
            offset: 0,
            peeked_offset: 0,
            peeked: None,
        }
    }

    fn offset(&self) -> usize {
        if self.peeked.is_some() {
            self.offset
        } else {
            self.peeked_offset
        }
    }

    fn next(&mut self) -> Result<Option<char>, I::Error> {
        if let Some(peeked) = self.peeked.take() {
            self.offset = self.peeked_offset;
            return peeked;
        }

        let result = self.input.next();
        if let Ok(Some(c)) = result {
            self.peeked_offset += c.len_utf8();
        }
        result
    }

    fn peek(&mut self) -> &Result<Option<char>, I::Error> {
        match self.peeked {
            Some(ref existing) => existing,
            None => {
                let next = self.next();
                self.peeked.insert(next)
            }
        }
    }

    fn next_if<F: FnOnce(char) -> bool>(&mut self, predicate: F) -> Result<Option<char>, I::Error> {
        match self.peek() {
            Ok(Some(c)) if predicate(*c) => {
                let c = *c;
                self.next()?;
                Ok(Some(c))
            }
            _ => Ok(None),
        }
    }
}

struct TokenBuilder<'cache> {
    string_cache: &'cache StringCache<'cache>,
    previous_offset: usize,
    tokens: Vec<(Token<'cache>, Range<usize>)>,
    unknown_buffer: String,
}

impl<'cache> TokenBuilder<'cache> {
    fn new(string_cache: &'cache StringCache<'cache>) -> Self {
        Self {
            string_cache,
            previous_offset: 0,
            tokens: Vec::new(),
            unknown_buffer: String::new(),
        }
    }

    fn append_unknown(&mut self, c: char) {
        self.unknown_buffer.push(c);
    }

    fn commit_unknown(&mut self) {
        if !self.unknown_buffer.is_empty() {
            let length = self.unknown_buffer.len();
            let start_offset = self.previous_offset;
            self.previous_offset += length;
            self.tokens.push((
                Token::Unknown(self.string_cache.store(&mut self.unknown_buffer)),
                start_offset..self.previous_offset,
            ));
        }
    }

    fn skip_char(&mut self, c: char) {
        self.commit_unknown();
        self.previous_offset += c.len_utf8();
    }

    fn commit(&mut self, token: Token<'cache>, offset: usize) {
        self.commit_unknown();
        debug_assert!(offset >= self.previous_offset);
        self.tokens.push((token, self.previous_offset..offset));
        self.previous_offset = offset;
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
    let mut input = Characters::new(source.into_input());
    let mut tokens = TokenBuilder::new(string_cache);
    let mut offsets = OffsetsBuilder::new();
    let mut buffer = String::new();

    // Read a UTF-8 BOM if it is present
    input.next_if(|c| c == '\u{FEFF}')?;

    while let Some(c) = input.next()? {
        // TODO: Define helper method to commit a buffer containing unknown chars.
        match c {
            '\r' | '\n' => {
                let offset = input.offset() - 1;
                tokens.skip_char(c);
                if c == '\r' {
                    if let Some(n) = input.next_if(|c| c == '\n')? {
                        tokens.skip_char(n);
                    }
                }
                offsets.new_line(offset);
            }
            '{' => tokens.commit(Token::OpenBracket, input.offset()),
            '}' => tokens.commit(Token::CloseBracket, input.offset()),
            ';' => tokens.commit(Token::Semicolon, input.offset()),
            '.' => {
                let mut has_chars = false;
                while let Some(l) = input.next_if(char::is_alphabetic)? {
                    buffer.push(l);
                    has_chars = true;
                }

                if has_chars {
                    tokens.commit(Token::Directive(string_cache.get_or_insert(&mut buffer)), input.offset());
                } else {
                    tokens.append_unknown(c);
                }
            }
            _ if c.is_whitespace() => tokens.skip_char(c),
            _ => tokens.append_unknown(c),
        }
    }

    tokens.commit_unknown();

    Ok(Output {
        tokens: tokens.tokens,
        strings: string_cache,
        offsets: offsets.finish(input.offset()),
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
