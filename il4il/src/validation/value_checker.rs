//! Provides functions for validating values and their expected types.

use crate::instruction::{value, Value};
use crate::type_system;
use crate::validation::type_resolver::IntoType;
use crate::validation::ModuleContents;
use error_stack::ResultExt;

/// The error type used when a value is invalid.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{value:?} is invalid")]
pub struct InvalidValueError {
    value: Value,
}

pub type Result = error_stack::Result<(), InvalidValueError>;

pub(crate) fn check_value<T: IntoType>(value: &Value, expected_type: T, contents: &ModuleContents) -> Result {
    let fail = || InvalidValueError { value: *value };
    let expected = expected_type.into_type(contents).change_context_lazy(fail)?;

    match value {
        Value::Constant(value::Constant::Integer(_)) => {
            if let type_system::Type::Integer(_) = expected {
                Ok(())
            } else {
                Err(error_stack::Report::new(fail())).attach_printable_lazy(|| format!("cannot use integer constant with {expected} type"))
            }
        }
        Value::Constant(value::Constant::Float(float_value)) => {
            if let type_system::Type::Float(float_type) = expected {
                if float_type.bit_width() == float_value.bit_width() {
                    Ok(())
                } else {
                    Err(error_stack::Report::new(fail())).attach_printable_lazy(|| {
                        format!(
                            "expected floating-point with bit width of {} type, but got {}",
                            float_type.bit_width(),
                            float_value.bit_width()
                        )
                    })
                }
            } else {
                Err(error_stack::Report::new(fail())).attach_printable_lazy(|| format!("cannot use float constant with {expected} type"))
            }
        }
    }
}

pub(crate) fn check_values_iter<'a, T, I>(values: I, contents: &ModuleContents) -> Result
where
    T: IntoType,
    I: IntoIterator<Item = (&'a Value, T)>,
{
    values
        .into_iter()
        .try_for_each(|(value, expected_type)| check_value(value, expected_type, contents))
}
