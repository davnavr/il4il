//! Provides functions to determine wheter two types are considered equal.

use crate::type_system::Type;
use crate::validation::ModuleContents;

/// Error type used when two types are not considered equal.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("expected {expected}, but got {actual}")]
pub struct TypeMismatchError {
    expected: Type,
    actual: Type,
}

impl TypeMismatchError {
    fn new(expected: Type, actual: Type) -> Self {
        Self { expected, actual }
    }
}

pub type Result = std::result::Result<(), TypeMismatchError>;

pub fn are_equal(expected: &Type, actual: &Type, _: &ModuleContents) -> Result {
    // TODO: When types contain indices to other types, check that they point to the same things.
    if expected != actual {
        return Err(TypeMismatchError::new(expected.clone(), actual.clone()));
    }

    Ok(())
}

//pub fn are_all_equal<'a, T>(types: T, contents: &ModuleContents) -> Result
//where
//    T: IntoIterator<Item = (&'a Type, &'a Type)>,
//    T::IntoIter: ExactSizeIterator,
//{
//    types
//        .into_iter()
//        .try_for_each(|(expected, actual)| are_equal(expected, actual, contents))
//}
