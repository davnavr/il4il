//! Functions for manipulating SAILAR identifier strings.

use crate::error::{self, Error, Message};
use crate::pointer::{self, Exposed};

pub use il4il::identifier::Identifier;

/// Creates an identifier string by copying from a sequence of bytes with the specified byte `length`. If the bytes are not valid UTF-8 or
/// any argument pointers are invalid, returns an error that can be disposed with [`il4il_error_dispose`].
///
/// The identifier can be disposed later with [`il4il_identifier_dispose`].
///
/// # Safety
///
/// Callers must ensure that the `contents` point to an allocation of at least `length` bytes.
///
/// # Panics
///
/// Panics if any [pointers are not valid](crate::pointer).
///
/// [`il4il_error_dispose`]: error::il4il_error_dispose
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_from_utf8<'a>(
    contents: *const u8,
    length: usize,
    identifier: Exposed<'a, &'a mut *mut Identifier>,
) -> Error {
    let create = || -> Result<_, Message> {
        let code_points = unsafe {
            // Safety: contents is assumed to be valid for length bytes
            pointer::as_slice(contents, length).expect("contents")
        };

        Ok(Box::into_raw(Box::new(<Identifier as std::str::FromStr>::from_str(
            std::str::from_utf8(code_points)?,
        )?)))
    };

    error::wrap_with_result(
        create,
        unsafe {
            // Safety: Caller is assumed to pass a valid pointer
            identifier.unwrap()
        }
        .expect("identifier"),
    )
}

/// Creates an identifier string from a sequence of UTF-16 code points.
///
/// See [`il4il_identifier_from_utf8`] for more information.
///
/// # Safety
///
/// Callers must ensure that the `contents` point to a valid allocation containing at least `count` code points.
///
/// # Panics
///
/// Panics if any [pointers are not valid](crate::pointer).
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_from_utf16<'a>(
    contents: *const u16,
    count: usize,
    identifier: Exposed<'a, &'a mut *mut Identifier>,
) -> Error {
    let create = || -> Result<_, Message> {
        let code_points = unsafe {
            // Safety: contents is assumed to be valid for length bytes
            pointer::as_slice(contents, count).expect("contents")
        };

        Ok(Box::into_raw(Box::new(Identifier::from_string(String::from_utf16(code_points)?)?)))
    };

    error::wrap_with_result(
        create,
        unsafe {
            // Safety: Caller is assumed to pass a valid pointer
            identifier.unwrap()
        }
        .expect("identifier"),
    )
}

/// Disposes an identifier string.
///
/// # Safety
///
/// Callers must ensure that the identifier has not already been disposed.
///
/// # Panics
///
/// Panics if the [`identifier` pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_dispose(identifier: Exposed<'static, Box<Identifier>>) {
    unsafe {
        // Safety: caller must ensure identifier is a valid pointer
        identifier.unwrap().expect("identifier");
    }
}

/// Gets the length, in bytes, of an identifier string.
///
/// # Safety
///
/// Callers must ensure that the identifier has not already been disposed.
///
/// # Panics
///
/// Panics if the [`identifier` pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_byte_length<'a>(identifier: Exposed<'a, &'a Identifier>) -> usize {
    unsafe {
        // Safety: caller is assumed to pass a valid pointer
        identifier.unwrap().expect("identifier").len()
    }
}

/// Copies the UTF-8 contents of an identifier string into a buffer.
///
/// # Safety
///
/// Callers must ensure that the identifier has not already been disposed, and that the buffer points to a valid allocation of the length
/// returned by [`il4il_identifier_byte_length`].
///
/// # Panics
///
/// Panics if any [pointer is not valid](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_identifier_copy_bytes_to<'a>(identifier: Exposed<'a, &'a Identifier>, buffer: *mut u8) {
    let id = unsafe {
        // Safety: identifier is assumed to be valid
        identifier.unwrap().expect("identifier")
    };

    let bytes: &'a mut [u8] = unsafe {
        // Buffer is assumed to be valid for the specified length.
        pointer::as_mut_slice(buffer, id.len()).expect("buffer")
    };

    bytes.copy_from_slice(id.as_bytes());
}
