//! Provides functions to examine validated IL4IL modules.

use crate::pointer::{self, Exposed};
use il4il::validation;

pub type Instance = validation::ValidModule<'static>;

/// Disposes a browser.
///
/// # Safety
///
/// Callers must ensure that the browser has not already been disposed.
///
/// # Panics
///
/// Panics if the [`browser` pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe fn il4il_browser_dispose(browser: Exposed<'static, Box<Instance>>) {
    unsafe {
        // Safety: Caller ensures browser is dereferenceable
        browser.unwrap().expect("browser");
    }
}

/// Gets the number of entries in the module's metadata sections.
///
/// # Safety
///
/// Callers must ensure that the browser has not been disposed.
///
/// # Panics
///
/// Panics if any [pointers are not valid](crate::pointer).
#[no_mangle]
pub unsafe fn il4il_browser_metadata_count<'a>(browser: Exposed<'a, &'a Instance>) -> usize {
    unsafe {
        // Safety: Caller ensures browser is dereferenceable
        browser.unwrap().expect("browser").contents().metadata.len()
    }
}

/// Copies the references to the module's metadata section contents to a buffer.
///
/// # Safety
///
/// Callers must ensure that the browser has not been disposed, and that the returned metadata references are only used for the lifetime of
/// the browser.
///
/// # Panics
///
/// Panics if any [pointers are not valid](crate::pointer).
#[no_mangle]
pub unsafe fn il4il_browser_metadata_copy_to<'a>(browser: Exposed<'a, &'a Instance>, buffer: *mut &'a crate::metadata::Metadata) {
    let metadata = unsafe {
        // Safety: Caller ensures browser is dereferenceable
        &browser.unwrap().expect("browser").contents().metadata
    };

    let destination = unsafe {
        // Safety: Caller ensures buffer is dereferenceable
        pointer::as_mut_slice(buffer, metadata.len()).expect("buffer")
    };

    for (data, dest) in metadata.iter().zip(destination) {
        *dest = data;
    }
}
