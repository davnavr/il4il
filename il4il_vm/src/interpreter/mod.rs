//! Contains the IL4IL bytecode interpreter.

mod error;

pub use error::{Error, ErrorKind};

pub mod call_stack;
pub mod value;

use crate::runtime::{self, Function};

pub type Result<T> = std::result::Result<T, Error>;

/// Encapsulates all state for a single thread of interpretation.
///
/// For simple scenarios, an [`Interpreter`] can be used to quickly evaluate the result of calling an IL4IL function.
///
/// For more complex situations, the [`host`] module is usually used to handle interpretation of IL4IL programs.
///
/// [`host`]: crate::host
pub struct Interpreter<'env> {
    runtime: &'env runtime::Runtime<'env>,
    call_stack: Vec<call_stack::Frame<'env>>,
}

impl<'env> Interpreter<'env> {
    pub fn initialize(runtime: &'env runtime::Runtime<'env>, entry_point: Function<'env>, arguments: Box<[value::Value]>) -> Self {
        Self {
            runtime,
            call_stack: vec![call_stack::Frame::new(runtime, entry_point, arguments)],
        }
    }

    /// Iterates over the frames in the interpreter's call stack, starting with the most recent frames first.
    pub fn iter_call_stack(&self) -> impl std::iter::ExactSizeIterator<Item = &call_stack::Frame<'env>> {
        self.call_stack.iter().rev()
    }

    pub fn runtime(&self) -> &'env runtime::Runtime<'env> {
        self.runtime
    }

    /// Interprets a single instruction.
    ///
    /// Returns `Ok(None)` if there are more instructions to execute and `Ok(Some)` if execution is complete.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] describing what went wrong.
    pub fn step(&mut self) -> Result<Option<Box<[value::Value]>>> {
        use il4il::instruction::Instruction;

        let current_frame = self.call_stack.last().ok_or_else(|| Error::new(ErrorKind::EndOfProgram))?;

        let return_values: Option<Box<[value::Value]>> = match current_frame.kind() {
            call_stack::FrameKind::Bytecode(code_frame) => match code_frame.advance() {
                Instruction::Unreachable => return Err(Error::new(ErrorKind::EncounteredUnreachable)),
                Instruction::Return(values) => {
                    let return_types = code_frame.block().body().result_types();

                    if return_types.len() != values.len() {
                        panic!("error kind for result count mismatch (expected {} values)", return_types.len());
                    }

                    Some(
                        return_types
                            .iter()
                            .zip(values.iter())
                            .map(|(value_type, value)| current_frame.create_value(value, value_type.as_type()))
                            .collect(),
                    )
                }
                bad => return Err(Error::new(ErrorKind::UnsupportedInstruction(bad.clone()))),
            },
            call_stack::FrameKind::Host(host_frame) => {
                let host_function = host_frame.function();
                let return_values = host_function
                    .invoke(current_frame.arguments(), self.runtime)
                    .map_err(|e| Error::new(ErrorKind::HostFunctionError(e)))?; // TODO: Incl stack trace.

                // TODO: Type check the return values.
                Some(return_values)
            }
        };

        if let Some(results) = return_values {
            self.call_stack.pop();
            if let Some(previous_frame) = self.call_stack.last() {
                todo!("insert registers containing results {:?}", previous_frame)
            } else {
                // Call stack is empty, return the results
                Ok(Some(results))
            }
        } else {
            // No return values, continue execution of function
            Ok(None)
        }
    }
}

impl std::fmt::Debug for Interpreter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[repr(transparent)]
        struct Frames<'a, 'b: 'a>(&'a [call_stack::Frame<'b>]);

        impl std::fmt::Debug for Frames<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_list().entries(self.0.iter().rev()).finish()
            }
        }

        f.debug_struct("Interpreter")
            .field("frames", &Frames(&self.call_stack))
            .finish_non_exhaustive()
    }
}
