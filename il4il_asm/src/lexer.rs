//! The IL4IL assembly lexer.

use logos::Logos;

/// The type of tokens used by the IL4IL assembler.
#[derive(Logos, Debug, PartialEq)]
pub enum Token<'input> {
    #[token("{")]
    OpenBracket,
    #[token("}")]
    CloseBracket,
    #[token(";")]
    Semicolon,
    #[regex(r"\.[a-zA-Z][a-zA-Z_0-9]*")]
    Directive(&'input str),
    #[error]
    #[regex(r"[ \t\n\r]+", logos::skip)] // Whitespace
    #[regex(r"//.*")] // Line comment
    Unknown,
}
