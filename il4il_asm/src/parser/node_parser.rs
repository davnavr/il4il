//! Low-level syntax node parser.

use crate::error::Error;
use crate::lexer;
use crate::syntax::structure;
use std::ops::Range;

pub(super) fn parse<'src>(
    tokens: &[(lexer::Token, Range<usize>)],
    offsets: &lexer::Offsets,
    errors: &mut Vec<Error>,
) -> structure::Tree<'src> {
    let mut contents = Vec::new();
    todo!();
    structure::Tree { contents }
}
