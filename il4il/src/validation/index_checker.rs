//! Provides functions for validating indices to content within a module.

use crate::index;
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

macro_rules! module_indexer {
    ($($vis:vis fn $name:ident($index_type:ty) -> $item_type:ty [$field:ident];)*) => {
        $($vis fn $name<'a>(
            contents: &'a crate::validation::ModuleContents,
            index: $index_type
        ) -> error_stack::Result<&'a $item_type, InvalidIndexError> {
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
    pub fn get_type(index::Type) -> crate::type_system::Type [types];
}
