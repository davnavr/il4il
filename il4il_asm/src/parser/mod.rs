//! Module for parsing IL4IL assembly.

use crate::error::Error;
use crate::lexer;
use crate::syntax;

mod node_parser;
mod tree_parser;

#[derive(Debug)]
pub struct Output<'cache> {
    pub(crate) offsets: lexer::Offsets,
    pub(crate) tree: syntax::tree::Root<'cache>,
}

impl<'cache> Output<'cache> {
    pub fn tree(&self) -> &syntax::tree::Root<'cache> {
        &self.tree
    }

    pub fn offsets(&self) -> &lexer::Offsets {
        &self.offsets
    }
}

pub fn parse<'cache>(inputs: crate::lexer::Output<'cache>, errors: &mut Vec<Error>) -> Output<'cache> {
    let tokens = inputs.tokens;
    let offsets = inputs.offsets;
    let structure = node_parser::parse(tokens, &offsets, errors);
    Output {
        tree: tree_parser::parse(structure, &offsets, errors),
        offsets,
    }
}
