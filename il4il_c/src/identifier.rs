//! Functions for manipulating SAILAR identifier strings.

use crate::error::{self, Message};
use crate::pointer;
use il4il::identifier::Identifier;
use std::str::FromStr;

/// Creates an identifier string by copying from a sequence of bytes with the specified byte `length`. If the bytes are not valid UTF-8 or
/// any argument pointers are invalid, returns `null` and an error that can be disposed with [`il4il_error_dispose`].
///
/// The identifier can be disposed later with [`il4il_identifier_dispose`].
///
/// # Safety
///
/// Callers must ensure that the `contents` and `error` [pointers are valid](mod@pointer#safety), and that `contents` points to an
/// allocation of at least `length` bytes.
///
/// # Panics
///
/// Panics if the [`error` pointer is not valid](#error::catch).
///
/// [`il4il_error_dispose`]: error::il4il_error_dispose
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_from_utf8(contents: *const u8, length: usize, error: *mut *mut Message) -> *mut Identifier {
    let create = || -> Result<_, Message> {
        let code_points = if length == 0 {
            Default::default()
        } else {
            unsafe {
                // Safety: contents are assumed to meet other pointer requirements.
                pointer::as_mut("contents", contents as *mut u8)?;
                // Safety: contents are assumed to be valid for length
                std::slice::from_raw_parts(contents, length)
            }
        };

        Ok(Box::into_raw(Box::new(Identifier::from_str(std::str::from_utf8(code_points)?)?)))
    };

    unsafe {
        // Safety: error is assumed to be dereferenceable.
        error::catch_or_else(create, std::ptr::null_mut, error)
    }
}

/// Disposes an identifier string.
///
/// # Safety
///
/// Callers must ensure that the identifier has not already been disposed.
///
/// # Panics
///
/// Panics if the identifier is not [a valid pointer](mod@pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_dispose(identifier: *mut Identifier) {
    unsafe {
        // Safety: Caller must ensure identifier is valid pointer.
        pointer::into_boxed("identifier", identifier).unwrap();
    }
}
