//! Module for parsing IL4IL assembly.

use crate::error::Error;
use crate::lexer;
use crate::syntax;

mod node_parser;
mod tree_parser;

#[derive(Debug)]
pub struct Output<'src> {
    pub(crate) offsets: lexer::Offsets,
    pub(crate) tree: syntax::tree::Root<'src>,
}

impl<'src> Output<'src> {
    pub fn tree(&self) -> &syntax::tree::Root<'src> {
        &self.tree
    }
}

pub fn parse<'src>(inputs: crate::lexer::Output<'src>, errors: &mut Vec<Error>) -> Output<'src> {
    let tokens = inputs.tokens;
    let offsets = inputs.offsets;
    let structure = node_parser::parse(&tokens, &offsets, errors);
    Output {
        tree: tree_parser::parse(structure, &offsets, errors),
        offsets,
    }
}
