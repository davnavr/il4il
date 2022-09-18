//! The IL4IL assembler takes a syntax tree and produces an in-memory representation of an IL4IL module.

use crate::error::Error;
use crate::syntax::tree;
use il4il::module;

pub type Output<'cache> = module::Module<'cache>;

pub fn assemble<'cache>(inputs: crate::parser::Output<'cache>, errors: &mut Vec<Error>) -> Output<'cache> {
    todo!()
}
