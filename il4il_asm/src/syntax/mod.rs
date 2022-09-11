//! Contains types to model the syntax tree of an IL4IL assembly program.

pub(crate) mod structure;
pub mod tree;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Located<N> {
    pub node: N,
    pub offsets: std::ops::Range<usize>,
}

impl<N> Located<N> {
    pub fn new(node: N, offsets: std::ops::Range<usize>) -> Self {
        Self { node, offsets }
    }
}
