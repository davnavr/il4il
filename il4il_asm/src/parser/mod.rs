//! Module for parsing IL4IL assembly.

use crate::error::{self, Error};
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

    fn push_error_at<M: error::Message>(&mut self, offsets: Range<usize>, message: M) {
        self.push_error(Error::new(self.offsets.get_location_range(offsets), message))
    }

    fn report_error<T>(&mut self, result: error::Result<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(e) => {
                self.push_error(e);
                None
            }
        }
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
