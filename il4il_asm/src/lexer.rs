//! The IL4IL assembly lexer.

use crate::location;
use logos::Logos;
use std::ops::Range;

#[derive(Logos)]
#[logos(extras = OffsetsBuilder)]
enum Tok<'src> {
    #[token("{")]
    OpenBracket,
    #[token("}")]
    CloseBracket,
    #[token(";")]
    Semicolon,
    #[regex(r"\.[a-zA-Z][a-zA-Z_0-9]*", |lex| &lex.slice()[1..])]
    Directive(&'src str),
    #[regex(r"\n|\r|(\r\n)")]
    Newline,
    #[error]
    #[regex(r"[ \t]+", logos::skip)]
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// The type of tokens used by the IL4IL assembler.
pub enum Token<'src> {
    OpenBracket,
    CloseBracket,
    Semicolon,
    Directive(&'src str),
    Unknown(&'src str),
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
    byte_length: usize,
    lines: Vec<Line>,
    previous_offset: usize,
    line_number: location::Number,
}

impl<'src> OffsetsBuilder {
    fn new(byte_length: usize) -> Self {
        Self {
            byte_length,
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

    fn finish(mut self) -> Offsets {
        match self.lines.last() {
            Some(last) if last.bytes.end < self.byte_length - 1 => self.new_line(self.byte_length),
            _ => (),
        }

        Offsets {
            byte_length: self.byte_length,
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
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }
}

#[derive(Clone, Debug, Default)]
pub struct Output<'src> {
    tokens: Vec<(Token<'src>, Range<usize>)>,
    offsets: Offsets,
}

impl<'src> Output<'src> {
    pub fn tokens(&self) -> &[(Token<'src>, Range<usize>)] {
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
pub fn tokenize(source: &str) -> Output<'_> {
    if source.is_empty() {
        return Default::default();
    }

    let mut tokens = Vec::new();
    let mut lexer = Tok::lexer_with_extras(source, OffsetsBuilder::new(source.len()));

    while let Some(tok) = lexer.next() {
        let offset = lexer.span();

        let actual_token = match tok {
            Tok::OpenBracket => Token::OpenBracket,
            Tok::CloseBracket => Token::CloseBracket,
            Tok::Semicolon => Token::Semicolon,
            Tok::Directive(name) => Token::Directive(name),
            Tok::Newline => {
                lexer.extras.new_line(offset.start);
                continue;
            }
            Tok::Unknown => Token::Unknown(&source[offset.clone()]),
        };

        tokens.push((actual_token, offset));
    }

    Output {
        tokens,
        offsets: lexer.extras.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location;

    #[test]
    fn simple_directive_produces_correct_output() {
        let output = tokenize("\n.section {\n}\n");

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
    }

    #[test]
    fn lines_are_correct_for_input_with_newline_only() {
        assert_eq!(
            tokenize("\n").offsets().lines(),
            &[Line::new(0..0, location::Number::new(1).unwrap()),]
        );
    }

    #[test]
    fn lines_are_correct_when_no_trailing_newline() {
        assert_eq!(
            tokenize("\n\n.section").offsets().lines(),
            &[
                Line::new(0..0, location::Number::new(1).unwrap()),
                Line::new(1..1, location::Number::new(2).unwrap()),
                Line::new(2..10, location::Number::new(3).unwrap()),
            ]
        );
    }
}
