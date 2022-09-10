//! Module for manipulation of interpreter threads

use crate::host::Host;
use crate::interpreter;

type Handle<'host> = std::thread::ScopedJoinHandle<'host, interpreter::Result<Box<[interpreter::value::Value]>>>;

/// Represents a thread containing an IL4IL bytecode [`Interpreter`].
///
/// [`Interpreter`]: crate::interpreter::Interpreter
pub struct InterpreterThread<'host, 'scope: 'host, 'env: 'scope> {
    host: &'host Host<'host, 'scope, 'env>,
    join_handle: Handle<'scope>,
}

impl<'host, 'scope: 'host, 'env: 'scope> InterpreterThread<'host, 'scope, 'env> {
    pub(super) fn new(
        host: &'host Host<'host, 'scope, 'env>,
        builder: std::thread::Builder,
        entry_point: crate::runtime::Function<'env>,
        arguments: Box<[interpreter::value::Value]>,
    ) -> std::io::Result<Self> {
        let mut interpreter = interpreter::Interpreter::<'env>::initialize(host.runtime, entry_point, arguments);

        let join_handle = builder.spawn_scoped(host.scope(), move || loop {
            match interpreter.step() {
                Ok(Some(values)) => return Ok(values),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        })?;

        Ok(Self { host, join_handle })
    }

    pub fn host(&self) -> &'host Host<'host, 'scope, 'env> {
        self.host
    }

    /// Blocks the current thread until the interpreter is finished executing.
    pub fn await_results_blocking(self) -> interpreter::Result<Box<[interpreter::value::Value]>> {
        match self.join_handle.join() {
            Ok(results) => results,
            Err(e) => std::panic::resume_unwind(e), // TODO: Figure out how to handle a thread panic
        }
    }
}
