//! Contains the IL4IL bytecode interpreter.

mod frame;
mod value;

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
