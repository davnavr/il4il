//! Provides error handling.

use crate::pointer::Exposed;

#[repr(transparent)]
pub struct Message(String);

impl<E: std::error::Error + Send + Sync + 'static> From<E> for Message {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

/// The result of fallible functions, can either be a pointer to an allocated [`Message`] value, or `null` to indicate that no error
/// occured.
pub type Error = Exposed<'static, Option<Box<Message>>>;

/// Unwraps a [`Result`] produced by a closure, returning any produced error [`Message`], or `null` if no error occured.
pub fn wrap<F: FnOnce() -> Result<(), Message>>(f: F) -> Error {
    f().err().map(Box::new).into()
}

/// Similar to [`wrap`], but writes any successfully produced value to a `destination`.
pub fn wrap_with_result<T, F: FnOnce() -> Result<T, Message>>(f: F, destination: &mut T) -> Error {
    wrap(|| match f() {
        Ok(value) => {
            *destination = value;
            Ok(())
        }
        Err(e) => Err(e),
    })
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
pub unsafe extern "C" fn il4il_error_dispose(message: Exposed<'static, Box<Message>>) {
    unsafe {
        // Safety: Provided by caller
        message.unwrap().expect("message");
    }
}
