//! Functions for manipulating IL4IL in-memory modules.

use crate::identifier::Identifier;
use crate::pointer::Exposed;
use il4il::binary::section::{self, Section};
use std::borrow::Cow;

pub type Instance = il4il::binary::Module<'static>;

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

/// Appends a module name to a metadata section within the module, copying from an identifier string.
///
/// # Safety
///
/// Callers must ensure that the module and name have not been disposed.
///
/// # Panics
///
/// Panics if the [a pointer is not valid](crate::pointer#safety).
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
