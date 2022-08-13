//! Helper functions for manipulating and validating pointers.
//!
//! # Safety
//!
//! The functions provided by this module check for the following conditions:
//!
//! - The `pointer` must not be [`null`](std::ptr::null).
//! - The `pointer` must be [properly aligned](std::ptr#alignment).
//!
//! Callers are responsible for ensuring that [the remaining rules regarding pointer validity](std::ptr#safety) are met.
//!
//! For additional information, see [the crate documentation](crate#safety).

#[derive(Debug, thiserror::Error)]
pub(crate) enum InvalidPointerKind {
    #[error("null")]
    Null,
    #[error("unaligned, expected alignment of {0}")]
    Unaligned(usize),
}

#[derive(Debug, thiserror::Error)]
#[error("{name} ({address:p}) was {kind}")]
pub(crate) struct InvalidPointerError {
    name: &'static str,
    address: usize,
    kind: InvalidPointerKind,
}

pub(crate) type Result<T> = std::result::Result<T, Box<InvalidPointerError>>;

/// Attempts to convert a raw pointer to a mutable reference.
///
/// # Safety
///
/// See the [module documentation](pointer).
pub(crate) unsafe fn as_mut<'a, T>(name: &'static str, pointer: *mut T) -> Result<&'a mut T> {
    let r: Option<&'a mut T> = unsafe {
        // Safety: pointer is assumed to be "dereferenceable"
        pointer.as_mut::<'a>()
    };

    let expected_alignment = std::mem::align_of::<T>();

    match r {
        Some(m) if pointer.align_offset(expected_alignment) == 0 => Ok(m),
        Some(_) => Err(Box::new(InvalidPointerError {
            name,
            address: pointer as usize,
            kind: InvalidPointerKind::Unaligned(expected_alignment),
        })),
        None => Err(Box::new(InvalidPointerError {
            name,
            address: pointer as usize,
            kind: InvalidPointerKind::Null,
        })),
    }
}

/// Attempts to convert a raw pointer to a mutable reference to a slice.
///
/// # Safety
///
/// See [`as_mut`] for more information.
pub(crate) unsafe fn as_mut_slice<'a, T>(name: &'static str, pointer: *mut T, count: usize) -> Result<&'a mut [T]> {
    if count == 0 {
        Ok(Default::default())
    } else {
        unsafe {
            // Safety: pointer is assumed to meet all requirements
            as_mut(name, pointer as *mut u8)?;
            // Safety: pointer is assumed to be valid for count
            Ok(std::slice::from_raw_parts_mut(pointer, count))
        }
    }
}

/// Attempts to create a [`Box`] from a raw pointer.
///
/// # Safety
///
/// Callers should ensure that the `pointer` was returned by a call to [`Box::into_raw`], and that this function is only called once for a
/// pointer to a particular allocation.
///
/// Additionally, callers must meet the same pointer validity rules of [`as_mut`](as_mut#safety).
pub(crate) unsafe fn into_boxed<T>(name: &'static str, pointer: *mut T) -> Result<Box<T>> {
    let result = unsafe {
        // Safety: Callers are responsible
        as_mut(name, pointer)
    };

    result.map(|_| unsafe {
        // Safety: Assumed to point to a box's contents
        Box::from_raw(pointer)
    })
}
