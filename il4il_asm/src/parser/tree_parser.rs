//! Turns a tree containing nodes into an abstract syntax tree which is the final output of the parsing process.

use crate::error::Error;
use crate::syntax::tree;

pub(super) fn parse<'src>(
    tree: crate::syntax::structure::Tree<'src>,
    offsets: &crate::lexer::Offsets,
    errors: &mut Vec<Error>,
) -> tree::Root<'src> {
    todo!("{:?} {:?} {:?}", tree, offsets, errors);
}
