//! Functions for manipulating IL4IL in-memory modules.

use crate::error::{self, Error};
use crate::identifier::Identifier;
use crate::pointer::Exposed;
use il4il::module::section::{self, Section};
use std::borrow::Cow;

pub type Instance = il4il::module::Module<'static>;

/// Creates a new module.
///
/// # Safety
///
/// Callers must ensure that the module is later disposed with [`il4il_module_dispose`].
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
/// Panics if the [`module` pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_module_dispose(module: Exposed<'static, Box<Instance>>) {
    unsafe {
        // Safety: module is assumed to be dereferenceable
        module.unwrap().expect("module");
    }
}

/// Performs validation on a module, and disposes the module. If an error occured, returns an [`Error`]; otherwise, returns `null`.
///
/// Callers should dispose the returned browser later by calling [`il4il_browser_dispose`](crate::browser::il4il_browser_dispose).
///
/// Note that validation techncally takes "ownership" of the underlying module, essentially meaning that the original `module`
/// pointer must no longer be used.
///
/// # Safety
///
/// Callers must ensure that the module has not already been disposed.
///
/// # Panics
///
/// Panics if a [pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_module_validate_and_dispose<'a>(
    module: Exposed<'static, Box<Instance>>,
    validated: Exposed<'a, &'a mut *mut crate::browser::Instance>,
) -> Error {
    error::wrap_with_result(
        || {
            let m = unsafe {
                // Safety: Caller ensures module is dereferenceable
                module.unwrap().expect("module")
            };

            Ok(Box::into_raw(Box::new(crate::browser::Instance::try_from(*m)?)))
        },
        unsafe {
            // Safety: Caller ensures this is dereferenceable
            validated.unwrap().expect("validated")
        },
    )
}

/// Appends a module name to a metadata section within the module, copying from an identifier string.
///
/// # Safety
///
/// Callers must ensure that the module and name have not been disposed.
///
/// # Panics
///
/// Panics if [a pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_module_add_metadata_name<'a>(module: Exposed<'a, &'a mut Instance>, name: Exposed<'a, &'a Identifier>) {
    let builder = unsafe {
        // Safety: module is assumed to be dereferenceable
        module.unwrap().expect("module")
    };

    let id = unsafe {
        // Safety: name is assumed to be dereferenceable
        name.unwrap().expect("name")
    };

    let sections = builder.sections_mut();
    let metadata = match sections.last_mut() {
        Some(Section::Metadata(md)) => md,
        _ => {
            sections.push(Section::Metadata(Vec::new()));
            if let Section::Metadata(md) = sections.last_mut().unwrap() {
                md
            } else {
                unreachable!()
            }
        }
    };

    metadata.push(section::Metadata::Name(Cow::Owned(id.clone())))
}

/// Given an identifier string containing a path, writes the binary contents of the module to the file. Any IO error that occured can be
/// disposed with [`il4il_error_dispose`].
///
/// # Safety
///
/// Callers must ensure that the module and path have not been disposed.
///
/// # Panics
///
/// Panics if [a pointer is not valid](crate::pointer#safety).
///
/// [`il4il_error_dispose`]: crate::error::il4il_error_dispose
#[no_mangle]
pub unsafe extern "C" fn il4il_module_write_binary_to_path<'a>(
    module: Exposed<'a, &'a Instance>,
    path: Exposed<'a, &'a Identifier>,
) -> Error {
    let mdle = unsafe {
        // Safety: caller is assumed to have passed a valid pointer
        module.unwrap().expect("module")
    };

    let identifier = unsafe {
        // Safety: caller is assumed to have passed a valid pointer
        path.unwrap().expect("path")
    };

    error::wrap(|| {
        mdle.write_to_path(identifier)?;
        Ok(())
    })
}
