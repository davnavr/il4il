//! Contains types modelling a low-level view of an IL4IL program.

use crate::cache::StringRef;
use crate::syntax::literal;
use crate::syntax::Located;
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
pub enum NodeKind<S: Deref<Target = str>> {
    Word(S),
    Directive(S),
}

impl<'str, S: StringRef<'str>> Display for NodeKind<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(word) => f.write_str(word),
            Self::Directive(name) => {
                f.write_char('.')?;
                f.write_str(name)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Attribute<S: Deref<Target = str>> {
    Word(S),
    String(literal::String<S>),
}

impl<'str, S: StringRef<'str>> Display for Attribute<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(word) => f.write_str(word),
            Self::String(str) => Display::fmt(&str, f),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum NodeContents<S: Deref<Target = str>> {
    Line(Vec<Located<Attribute<S>>>),
    /// A block containing content surrounded by curly brackets ('{' and '}').
    Block {
        attributes: Vec<Located<Attribute<S>>>,
        nodes: Vec<Located<Node<S>>>,
    },
    ///// A comma-separated list of items surrounded by square brackets ('[' and ']').
    //List
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Node<S: Deref<Target = str>> {
    pub kind: Located<NodeKind<S>>,
    pub contents: NodeContents<S>,
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Tree<S: Deref<Target = str>> {
    pub contents: Vec<Located<Node<S>>>,
}
