//! Contains the IL4IL bytecode interpreter.

mod error;
mod frame;
mod value;

pub use error::{Error, ErrorKind};
pub use frame::Frame;
pub use value::Value;

use crate::loader;
use crate::runtime;

/// Encapsulates all state for a single thread of interpretation.
pub struct Interpreter<'env> {
    runtime: &'env runtime::Runtime<'env>,
    call_stack: Vec<Frame<'env>>,
}

impl<'env> Interpreter<'env> {
    pub fn initialize(
        runtime: &'env runtime::Runtime<'env>,
        entry_point: &'env loader::function::Instantiation<'env>,
        arguments: Box<[Value]>,
    ) -> Self {
        Self {
            runtime,
            call_stack: vec![Frame::new(entry_point, arguments)],
        }
    }

    /// Iterates over the frames in the interpreter's call stack, starting with the most recent frames first.
    pub fn iter_call_stack(&self) -> impl std::iter::ExactSizeIterator<Item = &Frame<'env>> {
        self.call_stack.iter().rev()
    }

    pub fn runtime(&self) -> &'env runtime::Runtime<'env> {
        self.runtime
    }

    /// Interprets a single instruction.
    ///
    /// Returns Ok(None) if there are more instructions to execute and Ok(Some) if execution is complete.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] describing what went wrong.
    pub fn step(&mut self) -> Result<Option<Box<[Value]>>, Error> {
        use il4il::instruction::Instruction;

        let current_frame = self.call_stack.last_mut().ok_or_else(|| Error::new(ErrorKind::EndOfProgram))?;

        match current_frame.advance() {
            Instruction::Unreachable => return Err(Error::new(ErrorKind::EncounteredUnreachable)),
            Instruction::Return(values) => {
                let expected_types = current_frame.block().body().result_types();
                todo!("handle return")
            }
            bad => return Err(Error::new(ErrorKind::UnsupportedInstruction(bad.clone()))),
        }

        #[allow(unreachable_code)]
        Ok(None)
    }
}

impl std::fmt::Debug for Interpreter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[repr(transparent)]
        struct Frames<'a, 'b: 'a>(&'a [Frame<'b>]);

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
