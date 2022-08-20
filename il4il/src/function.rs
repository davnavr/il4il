//! Manipulation of IL4IL functions.

#![deny(unsafe_code)]

use crate::type_system;

/// Function definitions associate an IL4IL function body with a [`Signature`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Definition {
    /// An index to the function signature indicating the parameters and results of this function definition.
    pub signature: crate::index::FunctionSignature,
    //body: ,
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
