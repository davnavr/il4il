//! Contains the IL4IL bytecode interpreter.

use crate::loader;
use crate::runtime;

/// Encapsulates all state for a single thread of interpretation.
///
/// See [`Module::interpret_function_instantation`] or [`Module::interpret_entry_point`] for more information.
pub struct Interpreter<'env> {
    runtime: &'env runtime::Runtime<'env>,
    //call_stack: Vec<>,
}

impl<'env> Interpreter<'env> {
    pub fn initialize(runtime: &'env runtime::Runtime<'env>, entry_point: &'env loader::function::Instantiation<'env>) -> Self {
        Self {
            runtime
        }
    }

    pub fn runtime(&self) -> &'env runtime::Runtime<'env> {
        self.runtime
    }
}
