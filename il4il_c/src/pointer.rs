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

use std::marker::PhantomData;
use std::ptr::NonNull;

/// Error type used to indicate why a pointer is invalid.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidPointerKind {
    #[error("null")]
    Null,
    #[error("unaligned, expected alignment of {0}")]
    Unaligned(usize),
}

#[derive(Debug, thiserror::Error)]
#[error("{pointer:p} was {kind}")]
pub struct InvalidPointerError {
    pointer: *const u8,
    kind: InvalidPointerKind,
}

impl InvalidPointerError {
    fn new<T>(pointer: *const T, kind: InvalidPointerKind) -> Self {
        Self {
            pointer: pointer as *const u8,
            kind,
        }
    }
}

/// Trait for converting between pointers and references.
pub trait Pointer<'a>: Sized {
    type Raw;

    /// Converts a raw pointer into `Self`.
    ///
    /// # Errors
    ///
    /// Returns an error if a pointer is invalid.
    ///
    /// # Safety
    ///
    /// Callers must uphold [all rules regarding pointer validity](crate#safety).
    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError>;

    /// Converts `self` into a raw pointer.
    fn into_raw(self) -> Self::Raw;
}

impl<'a, T> Pointer<'a> for Option<NonNull<T>> {
    type Raw = *mut T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        if let Some(pointer) = NonNull::new(value) {
            let expected_alignment = std::mem::align_of::<T>();

            if pointer.as_ptr().align_offset(expected_alignment) != 0 {
                return Err(InvalidPointerError::new(
                    pointer.as_ptr(),
                    InvalidPointerKind::Unaligned(expected_alignment),
                ));
            }

            Ok(Some(pointer))
        } else {
            Ok(None)
        }
    }

    fn into_raw(self) -> Self::Raw {
        if let Some(pointer) = self {
            pointer.as_ptr()
        } else {
            std::ptr::null_mut()
        }
    }
}

impl<'a, T> Pointer<'a> for Option<&'a T> {
    type Raw = *const T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        let non_null = unsafe {
            // Safety: value is assumed to meet other pointer requirements
            <Option<NonNull<T>> as Pointer<'a>>::from_raw(value as *mut T)?
        };

        match non_null {
            Some(pointer) => unsafe {
                // Safety: callers uphold other pointer rules
                Ok(Some(pointer.as_ref()))
            },
            None => Ok(None),
        }
    }

    fn into_raw(self) -> Self::Raw {
        self.map(NonNull::from).into_raw() as *const T
    }
}

impl<'a, T> Pointer<'a> for Option<&'a mut T> {
    type Raw = *mut T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        let non_null = unsafe {
            // Safety: value is assumed to meet other pointer requirements
            <Option<NonNull<T>> as Pointer<'a>>::from_raw(value)?
        };

        match non_null {
            Some(mut pointer) => unsafe {
                // Safety: callers uphold other pointer rules
                Ok(Some(pointer.as_mut()))
            },
            None => Ok(None),
        }
    }

    fn into_raw(self) -> Self::Raw {
        self.map(NonNull::from).into_raw()
    }
}

impl<'a, T> Pointer<'a> for &'a mut T {
    type Raw = *mut T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        let pointer = unsafe {
            // Safety: callers uphold other pointer rules
            <Option<&'a mut T> as Pointer<'a>>::from_raw(value)?
        };

        pointer.ok_or_else(|| InvalidPointerError::new(value, InvalidPointerKind::Null))
    }

    fn into_raw(self) -> Self::Raw {
        self as *mut T
    }
}

impl<'a, T> Pointer<'a> for &'a T {
    type Raw = *const T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        let pointer = unsafe {
            // Safety: callers uphold other pointer rules
            <Option<&'a T> as Pointer<'a>>::from_raw(value)?
        };

        pointer.ok_or_else(|| InvalidPointerError::new(value, InvalidPointerKind::Null))
    }

    fn into_raw(self) -> Self::Raw {
        self as *const T
    }
}

impl<T> Pointer<'static> for Box<T> {
    type Raw = *mut T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        unsafe {
            // Safety: value is assumed to result in a valid pointer
            <&mut T as Pointer>::from_raw(value)?;
        }

        unsafe {
            // Safety: Any validation checks are performed above
            Ok(Box::from_raw(value))
        }
    }

    fn into_raw(self) -> Self::Raw {
        Box::into_raw(self)
    }
}

impl<T> Pointer<'static> for Option<Box<T>> {
    type Raw = *mut T;

    unsafe fn from_raw(value: Self::Raw) -> Result<Self, InvalidPointerError> {
        let non_null = unsafe {
            // Safety: other pointer rules are assumed to be met.
            <Option<NonNull<T>> as Pointer>::from_raw(value)?
        };

        Ok(non_null.map(|pointer| unsafe {
            // Safety: Any validation checks are performed above
            Box::from_raw(pointer.as_ptr())
        }))
    }

    fn into_raw(self) -> Self::Raw {
        match self {
            None => std::ptr::null_mut(),
            Some(b) => Box::into_raw(b),
        }
    }
}

/// Wrapper type for pointers used in `extern` functions.
#[derive(Debug)]
#[repr(transparent)]
pub struct Exposed<'a, P: Pointer<'a>> {
    raw: P::Raw,
    _phantom: PhantomData<P>,
}

impl<'a, P: Pointer<'a>> Exposed<'a, P> {
    /// Attempts to convert from a raw pointer.
    ///
    /// See [`Pointer::from_raw`] for more information.
    pub(crate) unsafe fn unwrap(self) -> Result<P, InvalidPointerError> {
        unsafe {
            // Safety: Caller is responsible
            P::from_raw(self.raw)
        }
    }

    /// Wraps a type that can be represented by a raw pointer.
    pub fn wrap(value: P) -> Self {
        Self {
            raw: P::into_raw(value),
            _phantom: PhantomData,
        }
    }
}

impl<'a, P: Pointer<'a>> From<P> for Exposed<'a, P> {
    fn from(value: P) -> Self {
        Self::wrap(value)
    }
}

pub(crate) unsafe fn as_mut_slice<'a, T>(pointer: *mut T, length: usize) -> Result<&'a mut [T], InvalidPointerError> {
    if length == 0 {
        Ok(Default::default())
    } else {
        unsafe {
            // Safety: caller is responsible
            <&'a mut T as Pointer<'a>>::from_raw(pointer)?;

            // Safety: validation is performed above
            Ok(std::slice::from_raw_parts_mut(pointer, length))
        }
    }
}

pub(crate) unsafe fn as_slice<'a, T>(pointer: *const T, length: usize) -> Result<&'a [T], InvalidPointerError> {
    unsafe {
        // Safety: caller is responsible
        as_mut_slice(pointer as *mut T, length).map(|slice| slice as &'a [T])
    }
}
