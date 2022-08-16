//! Provides error handling.

use crate::pointer::Exposed;

#[repr(transparent)]
pub struct Message(String);

impl Message {
    pub(crate) fn into_string(self) -> String {
        self.0
    }
}

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

/// Gets the length, in bytes, of an error's message string.
///
/// # Safety
///
/// Callers must ensure that the message has not already been disposed.
///
/// # Panics
///
/// Panics if the message is not a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn il4il_error_message_length<'a>(message: Exposed<'a, &'a Message>) -> usize {
    unsafe {
        // Safety: Provided by caller
        message.unwrap().expect("message").0.len()
    }
}

/// Copies the UTF-8 error message contents to a buffer.
///
/// # Safety
///
/// Callers must ensure that the message has not already been disposed and that the buffer points to a valid allocation of the correct
/// length.
///
/// # Panics
///
/// Panics if an [invalid pointer is detected](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_error_message_copy_to<'a>(message: Exposed<'a, &'a Message>, buffer: *mut u8) {
    let msg = unsafe {
        // Safety: message is assumed to be valid
        message.unwrap().expect("message")
    };

    let bytes: &'a mut [u8] = unsafe {
        // Buffer is assumed to be valid for the specified length.
        crate::pointer::as_mut_slice(buffer, msg.0.len()).expect("buffer")
    };

    bytes.copy_from_slice(msg.0.as_bytes());
}

/// Creates an error message from a sequence of UTF-16 code points. The message can be disposed later by calling [`il4il_error_dispose`].
///
/// This function is useful for allocating errors when the IL4IL C API needs to call a provided callback function that can fail.
///
/// # Safety
///
/// Callers must ensure that the contents is a valid pointer for `count` code points.
///
/// # Panics
///
/// Panics if an [invalid pointer is detected](crate::pointer#safety).
#[no_mangle]
pub unsafe extern "C" fn il4il_error_message_from_utf16(contents: *const u16, count: usize) -> Error {
    wrap(|| {
        let code_points = unsafe {
            // Safety: contents is assumed to be valid for the specified number of code points
            crate::pointer::as_slice(contents, count).expect("contents")
        };

        Err(Message(String::from_utf16_lossy(code_points)))
    })
}
