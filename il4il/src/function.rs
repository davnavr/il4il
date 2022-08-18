//! Manipulation of IL4IL functions.

#![deny(unsafe_code)]

use crate::type_system;

/// Function signatures specify the parameter types and return types of functions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Signature {
    types: Box<[type_system::Reference]>,
    return_type_count: usize,
}

impl Signature {
    pub fn into_types(self) -> Box<[type_system::Reference]> {
        self.types
    }
}
