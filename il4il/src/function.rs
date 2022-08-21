//! Manipulation of IL4IL functions.

#![deny(unsafe_code)]

use crate::index;
use crate::instruction;
use crate::type_system;

/// Iterates over the basic blocks of a function [`Body`].
///
/// See also the [`Body::iter_blocks`] function.
#[derive(Clone)]
pub struct Blocks<'body> {
    entry: Option<&'body instruction::Block>,
    others: std::slice::Iter<'body, instruction::Block>,
}

impl<'body> Iterator for Blocks<'body> {
    type Item = &'body instruction::Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.entry.take().or_else(|| self.others.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.len();
        (count, Some(count))
    }
}

impl ExactSizeIterator for Blocks<'_> {
    fn len(&self) -> usize {
        self.others.len() + if self.entry.is_some() { 1 } else { 0 }
    }
}

impl std::iter::FusedIterator for Blocks<'_> {}

/// A function body consists of a list of basic blocks and specifies the types of all inputs, temporary registers, and results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Body {
    entry_block: instruction::Block,
    other_blocks: Box<[instruction::Block]>,
}

impl Body {
    pub fn new(entry_block: instruction::Block, other_blocks: Box<[instruction::Block]>) -> Self {
        Self { entry_block, other_blocks }
    }

    pub fn entry_block(&self) -> &instruction::Block {
        &self.entry_block
    }

    pub fn other_blocks(&self) -> &[instruction::Block] {
        &self.other_blocks
    }

    pub fn iter_blocks(&self) -> Blocks<'_> {
        Blocks {
            entry: Some(&self.entry_block),
            others: self.other_blocks.iter(),
        }
    }
}

/// Function definitions associate an IL4IL function body with a [`Signature`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Definition {
    /// An index to the function signature indicating the parameters and results of this function definition.
    pub signature: index::FunctionSignature,
    pub body: index::FunctionBody,
}

impl Definition {
    pub fn new(signature: index::FunctionSignature, body: index::FunctionBody) -> Self {
        Self { signature, body }
    }
}

/// Function signatures specify the parameter types and result types of functions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Signature {
    types: Box<[type_system::Reference]>,
    result_type_count: usize,
}

impl Signature {
    /// Creates a function signature with the specified result types followed by the parameter types.
    ///
    /// # Panics
    ///
    /// Panics if the specified number of result types exceeds the total number of types.
    pub fn from_types<T: Into<Box<[type_system::Reference]>>>(types: T, result_type_count: usize) -> Self {
        let signature_types = types.into();
        assert!(result_type_count <= signature_types.len());
        Self {
            types: signature_types,
            result_type_count,
        }
    }

    pub fn result_type_count(&self) -> usize {
        self.result_type_count
    }

    pub fn parameter_type_count(&self) -> usize {
        self.types.len() - self.result_type_count
    }

    pub fn result_types(&self) -> &[type_system::Reference] {
        &self.types[0..self.result_type_count]
    }

    pub fn parameter_types(&self) -> &[type_system::Reference] {
        &self.types[self.result_type_count..]
    }

    pub(crate) fn all_types(&self) -> &[type_system::Reference] {
        &self.types
    }

    pub fn into_types(self) -> Box<[type_system::Reference]> {
        self.types
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Template {
    Definition(usize),
    //Import(),
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct TemplateLookup {
    templates: Vec<Template>,
}

impl TemplateLookup {
    pub(crate) fn reserve(&mut self, capacity: usize) {
        self.templates.reserve(capacity)
    }

    pub(crate) fn insert(&mut self, template: Template) {
        self.templates.push(template)
    }

    pub fn get_template(&self, index: crate::index::FunctionTemplate) -> Option<&Template> {
        self.templates.get(usize::from(index))
    }

    pub fn iter_templates(&self) -> impl std::iter::ExactSizeIterator<Item = &Template> {
        self.templates.iter()
    }
}

impl std::fmt::Debug for TemplateLookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter_templates()).finish()
    }
}
