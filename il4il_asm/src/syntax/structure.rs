//! Contains types modelling a low-level view of an IL4IL program.

use crate::syntax::Located;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub enum NodeKind<'src> {
    Word(&'src str),
    Directive(&'src str),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Attribute<'src> {
    Word(&'src str),
}

impl Display for Attribute<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(word) => f.write_str(word),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum NodeContents<'src> {
    Line(Vec<Located<Attribute<'src>>>),
    /// A block containing content surrounded by curly brackets ('{' and '}').
    Block {
        attributes: Vec<Located<Attribute<'src>>>,
        nodes: Vec<Located<Node<'src>>>,
    },
    ///// A comma-separated list of items surrounded by square brackets ('[' and ']').
    //List
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Node<'src> {
    pub kind: Located<NodeKind<'src>>,
    pub contents: NodeContents<'src>,
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Tree<'src> {
    pub contents: Vec<Located<Node<'src>>>,
}
