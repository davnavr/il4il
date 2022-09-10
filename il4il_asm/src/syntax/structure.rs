//! Contains types modelling a low-level view of an IL4IL program.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeKind<'src> {
    Word(&'src str),
    Directive(&'src str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Atom<'src> {
    Word(&'src str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeContents<'src> {
    Line(Vec<Atom<'src>>),
    Block {
        attributes: Vec<Atom<'src>>,
        nodes: Vec<Node<'src>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Node<'src> {
    pub kind: NodeKind<'src>,
    pub contents: NodeContents<'src>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Tree<'src> {
    pub contents: Vec<Node<'src>>,
}
