//! Module for parsing IL4IL assembly.

use crate::error::Error;
use crate::lexer;
use crate::syntax;
use std::ops::Range;

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

#[derive(Debug)]
struct Context<'a> {
    offsets: &'a lexer::Offsets,
    errors: &'a mut Vec<Error>,
}

impl<'a> Context<'a> {
    fn offsets(&self) -> &lexer::Offsets {
        self.offsets
    }

    fn push_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn push_error_at<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result + 'static>(&mut self, offsets: Range<usize>, message: F) {
        self.push_error(Error::new(self.offsets.get_location_range(offsets), message))
    }

    fn push_error_string_at(&mut self, offsets: Range<usize>, message: String) {
        self.push_error(Error::from_string(self.offsets.get_location_range(offsets), message))
    }

    fn push_error_str_at(&mut self, offsets: Range<usize>, message: &'static str) {
        self.push_error(Error::from_str(self.offsets.get_location_range(offsets), message))
    }
}

pub fn parse<'cache>(inputs: crate::lexer::Output<'cache>, errors: &mut Vec<Error>) -> Output<'cache> {
    let tokens = inputs.tokens;
    let mut context = Context {
        offsets: &inputs.offsets,
        errors,
    };

    let structure = node_parser::parse(tokens, &mut context);

    Output {
        tree: tree_parser::parse(structure, &mut context),
        offsets: inputs.offsets,
    }
}
