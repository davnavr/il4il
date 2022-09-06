//! Module for manipulation of interpreter threads

use crate::host::Host;
use crate::interpreter;

type Handle<'host> = std::thread::ScopedJoinHandle<'host, interpreter::Result<Box<[interpreter::Value]>>>;

/// Represents a thread containing an IL4IL bytecode [`Interpreter`].
///
/// [`Interpreter`]: crate::interpreter::Interpreter
pub struct InterpreterThread<'host, 'parent: 'host> {
    host: &'host Host<'host, 'parent>,
    handle: Handle<'host>,
}

impl<'host, 'parent: 'host> InterpreterThread<'host, 'parent> {
    pub(super) fn new(
        host: &'host Host<'host, 'parent>,
        builder: std::thread::Builder,
        entry_point: &'host interpreter::Function<'host>,
        arguments: Box<[interpreter::Value]>,
    ) -> std::io::Result<Self> {
        let mut interpreter = interpreter::Interpreter::initialize(&host.runtime, entry_point, arguments);

        let handle = builder.spawn_scoped(host.scope(), move || loop {
            match interpreter.step() {
                Ok(Some(values)) => return Ok(values),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        })?;

        Ok(Self { host, handle })
    }

    pub fn host(&self) -> &'host Host<'host, 'parent> {
        self.host
    }

    /// Blocks the current thread until the interpreter is finished executing.
    pub fn await_results_blocking(self) -> interpreter::Result<Box<[interpreter::Value]>> {
        match self.handle.join() {
            Ok(results) => results,
            Err(e) => std::panic::resume_unwind(e), // TODO: Figure out how to handle a thread panic
        }
    }
}
