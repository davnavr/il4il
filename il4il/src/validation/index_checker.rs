//! Provides functions for validating indices to content within a module.

use crate::index;
use crate::validation::ModuleContents;
use std::fmt::{Display, Formatter};

/// The error type used when an index to content within a module is invalid.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub struct InvalidIndexError {
    kind: &'static str,
    index: usize,
    maximum: Option<usize>,
}

impl InvalidIndexError {
    pub(crate) fn new<S: index::IndexSpace>(index: index::Index<S>, maximum: Option<usize>) -> Self {
        Self {
            kind: S::name(),
            index: usize::from(index),
            maximum,
        }
    }
}

impl Display for InvalidIndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} index #{} is out of bounds", self.kind, self.index)?;
        if let Some(maximum) = self.maximum {
            write!(f, "maximum valid index is #{}", maximum)?;
        }
        Ok(())
    }
}

pub type Result<T> = error_stack::Result<T, InvalidIndexError>;

macro_rules! module_indexer {
    ($($vis:vis fn $name:ident($index_type:ty) -> $item_type:ty [$field:ident];)*) => {
        $($vis fn $name<'a>(index: $index_type, contents: &'a ModuleContents) -> Result<&'a $item_type> {
            if let Some(item) = contents.$field.get(usize::from(index)) {
                Ok(item)
            } else {
                let count = contents.$field.len();
                error_stack::bail!(InvalidIndexError::new(index, if count == 0 { None } else { Some(count - 1) }))
            }
        })*
    };
}

module_indexer! {
    pub(crate) fn get_type(index::Type) -> crate::type_system::Type [types];

    pub(crate) fn get_function_signatures(index::FunctionSignature) -> crate::function::Signature [function_signatures];

    pub(crate) fn get_function_body(index::FunctionBody) -> crate::function::Body [function_bodies];
}

pub fn get_function_template<'a>(index: index::FunctionTemplate, contents: &'a ModuleContents) -> Result<&'a crate::function::Template> {
    if let Some(item) = contents.function_templates.get_template(index) {
        Ok(item)
    } else {
        let count = contents.function_templates.count();
        error_stack::bail!(InvalidIndexError::new(index, if count == 0 { None } else { Some(count - 1) }))
    }
}
