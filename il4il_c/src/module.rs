//! Functions for manipulating IL4IL in-memory modules.

use crate::error::{self, Message};
use crate::pointer;

pub type Instance = il4il::binary::Module<'static>;

/// Creates a new module.
///
/// # Safety
///
/// Callers must ensure that the module is late disposed with [`il4il_module_dispose`].
#[no_mangle]
pub unsafe extern "C" fn il4il_module_create() -> *mut Instance {
    Box::into_raw(Box::new(Instance::new()))
}

/// Disposes a module.
///
/// # Safety
///
/// Callers must ensure that the module has not already been disposed.
///
/// # Panics
///
/// Panics if the [`error` pointer is not valid](#error::catch).
#[no_mangle]
pub unsafe extern "C" fn il4il_module_dispose(module: *mut Instance, error: *mut *mut Message) {
    unsafe {
        // Safety: error is assumed to be dereferenceable.
        error::catch(
            || {
                // Safety: Caller must ensure module is valid pointer.
                Ok(pointer::into_boxed("module", module)?)
            },
            error,
        );
    }
}

/// Appends a metadata section to a module.
///
/// # Safety
///
/// Callers must ensure that the metadata pointer is no longer used after this function is called.
///
/// # Panics
///
/// Panics if the [`error` pointer is not valid](#error::catch).
#[no_mangle]
pub unsafe extern "C" fn il4il_module_append_metadata(
    module: *mut Instance,
    metadata: *mut crate::metadata::Builder,
    error: *mut *mut Message,
) {
    todo!()
}
