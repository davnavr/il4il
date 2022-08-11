//! Provides error handling.

use crate::pointer;

#[repr(transparent)]
pub struct Message(String);

impl<E: std::error::Error + Send + Sync + 'static> From<E> for Message {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

/// Unwraps a [`Result`] produced by a closure, and writes any error message to a pointer.
///
/// # Safety
///
/// Callers are responsible for ensuring that the error message pointer is [*dereferenceable*](std::ptr#safety)
///
/// # Panics
///
/// Panics if the error message pointer is not valid.
pub unsafe fn catch<T, F: FnOnce() -> Result<T, Message>>(f: F, error: *mut *mut Message) -> Option<T> {
    match f() {
        Ok(value) => Some(value),
        Err(message) => {
            let destination = unsafe {
                // Safety: Caller ensures error pointer is valid
                pointer::as_mut("error", error).unwrap()
            };

            *destination = Box::into_raw(Box::new(message));
            None
        }
    }
}

/// Unwraps a [`Result`] produced by a closure, writing any error message to a pointer and returning an alternative value.
///
/// # Safety
///
/// See the [`catch`](catch#safety) function.
///
/// # Panics
///
/// See [`catch`](catch#panics).
pub unsafe fn catch_or_else<T, F, E>(f: F, e: E, error: *mut *mut Message) -> T
where
    F: FnOnce() -> Result<T, Message>,
    E: FnOnce() -> T,
{
    unsafe {
        // Safety: Caller is responsible
        catch::<T, F>(f, error)
    }
    .unwrap_or_else(e)
}

/// Unwraps a [`Result`] produced by a closure, writing any error message to a pointer and returning [`Default::default()`].
///
/// # Safety
///
/// See the [`catch`](catch#safety) function.
///
/// # Panics
///
/// See [`catch`](catch#panics).
pub unsafe fn catch_or_default<T: Default, F: FnOnce() -> Result<T, Message>>(f: F, error: *mut *mut Message) -> T {
    unsafe {
        // Safety: Caller is responsible
        catch_or_else::<T, F, _>(f, Default::default, error)
    }
}

/// Frees the memory associated with the error message.
///
/// # Safety
///
/// Callers must ensure that the message has not already been disposed.
///
/// # Panics
///
/// Panics if the message is not a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn il4il_error_dispose(message: *mut Message) {
    unsafe {
        // Safety: Provided by caller
        pointer::into_boxed("message", message).unwrap();
    }
}
