//! Provides functions for validating values and their expected types.

use crate::instruction::{value, Value};
use crate::type_system;
use crate::validation::ModuleContents;
use error_stack::ResultExt;

/// The error type used when a value is invalid.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{value:?} is invalid")]
pub struct InvalidValueError {
    value: Value,
}

pub type Result = error_stack::Result<(), InvalidValueError>;

pub(crate) trait IntoType {
    type Error: error_stack::Context;

    fn into_type(self, contents: &ModuleContents) -> error_stack::Result<type_system::Type, Self::Error>;
}

impl IntoType for type_system::Type {
    type Error = std::convert::Infallible;

    fn into_type(self, _: &ModuleContents) -> error_stack::Result<type_system::Type, Self::Error> {
        Ok(self)
    }
}

impl IntoType for &type_system::Reference {
    //type Error = crate::validation::index_checker::InvalidIndexError;
    type Error = crate::validation::error::InvalidIndexError;

    fn into_type(self, contents: &ModuleContents) -> error_stack::Result<type_system::Type, Self::Error> {
        match self {
            type_system::Reference::Inline(ty) => Ok(*ty),
            type_system::Reference::Index(index) => todo!("index the module's type section"),
        }
    }
}

pub(crate) fn check_value<T: IntoType>(value: &Value, expected_type: T, contents: &ModuleContents) -> Result {
    let fail = || InvalidValueError { value: value.clone() };
    let expected = expected_type.into_type(contents).change_context_lazy(fail)?;

    match value {
        Value::Constant(value::Constant::Integer(_)) => {
            if let type_system::Type::Integer(_) = expected {
                Ok(())
            } else {
                Err(error_stack::Report::new(fail()).attach_printable(format!("cannot use integer constant with {expected} type")))
            }
        }
        Value::Constant(value::Constant::Float(float_value)) => {
            if let type_system::Type::Float(float_type) = expected {
                if float_type.bit_width() == float_value.bit_width() {
                    Ok(())
                } else {
                    Err(error_stack::Report::new(fail()).attach_printable(format!(
                        "expected floating-point with bit width of {} type, but got {}",
                        float_type.bit_width(),
                        float_value.bit_width()
                    )))
                }
            } else {
                Err(error_stack::Report::new(fail()).attach_printable(format!("cannot use float constant with {expected} type")))
            }
        }
    }
}

pub(crate) fn check_values_iter<'a, T, I>(values: I, contents: &ModuleContents) -> Result
where
    T: IntoType,
    I: IntoIterator<Item = (&'a Value, T)>,
{
    values.into_iter().try_for_each(|(value, expected_type)| check_value(value, expected_type, contents))
}
