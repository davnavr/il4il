//! Contains types to model the [abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) of an IL4IL assembly program.
//!
//! At this level, each node of the tree corresponds to content in the output module (e.g. a module section, type signatures, code).

use crate::syntax::Located;

#[derive(Clone, Debug)]
pub enum TopLevelDirective<'src> {
    Placeholder(&'src ()),
}

/// The root of the abstract syntax tree.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Root<'src> {
    pub directives: Box<[Located<TopLevelDirective<'src>>]>,
}
