//! Contains the [`Value`] struct.

use crate::loader::types::{self, TypeKind};
use std::num::NonZeroUsize;

pub use crate::runtime::configuration::Endianness;
pub use il4il::instruction::value::{Constant, ConstantInteger};

const POINTER_SIZE: usize = std::mem::size_of::<*const u8>();

#[derive(Clone, Copy)]
union Bits {
    inlined: [u8; POINTER_SIZE],
    allocated: *mut u8,
}

/// Represents a value, which is just a bunch of bytes.
#[derive(Clone)]
pub struct Value {
    length: NonZeroUsize,
    bits: Bits,
}

impl Value {
    /// Initializes a value of the specified byte width, with all bits set to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il_vm::interpreter::Value;
    /// assert_eq!(Value::zero(std::num::NonZeroUsize::new(5).unwrap()).byte_width().get(), 5);
    /// ```
    pub fn zero(byte_width: NonZeroUsize) -> Self {
        Self {
            length: byte_width,
            bits: if byte_width.get() <= POINTER_SIZE {
                Bits {
                    inlined: [0u8; POINTER_SIZE],
                }
            } else {
                let mut allocation = vec![0u8; byte_width.get()].into_boxed_slice();
                let pointer = allocation.as_mut_ptr();
                std::mem::forget(allocation);
                Bits { allocated: pointer }
            },
        }
    }

    /// Creates a value from the given bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il_vm::interpreter::Value;
    /// assert!(matches!(Value::from_bytes(&[1, 2, 3]), Some(value) if value.as_bytes() == &[1, 2, 3]));
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let length = NonZeroUsize::new(bytes.len())?;
        Some(Self {
            length,
            bits: if length.get() <= POINTER_SIZE {
                let mut inlined = [0u8; POINTER_SIZE];
                (&mut inlined[0..length.get()]).copy_from_slice(bytes);
                Bits { inlined }
            } else {
                let mut allocation = Box::<[u8]>::from(bytes);
                let pointer = allocation.as_mut_ptr();
                std::mem::forget(allocation);
                Bits { allocated: pointer }
            },
        })
    }

    /// Creates a value of a specified byte width, consisting of all bytes set to a `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il_vm::interpreter::Value;
    /// assert_eq!(Value::with_byte(1u8, std::num::NonZeroUsize::new(4).unwrap()).as_bytes(), &[1u8, 1, 1, 1]);
    /// ```
    pub fn with_byte(value: u8, byte_width: NonZeroUsize) -> Self {
        let mut bits = Self::zero(byte_width);
        bits.as_bytes_mut().fill(value);
        bits
    }

    /// Creates a value from a boxed slice of bytes.
    pub fn from_boxed_bytes(mut bytes: Box<[u8]>) -> Option<Self> {
        let length = NonZeroUsize::new(bytes.len())?;
        Some(Self {
            length,
            bits: if length.get() <= POINTER_SIZE {
                let mut inlined = [0u8; POINTER_SIZE];
                (&mut inlined[0..length.get()]).copy_from_slice(&bytes);
                Bits { inlined }
            } else {
                let allocated = bytes.as_mut_ptr();
                std::mem::forget(bytes);
                Bits { allocated }
            },
        })
    }

    pub(crate) fn from_constant_value<'env>(value: &Constant, value_type: &'env types::Type<'env>, endianness: Endianness) -> Self {
        match value_type.kind() {
            TypeKind::Integer(integer_type) => match value {
                Constant::Integer(integer_value) => {
                    let byte_width = NonZeroUsize::from(integer_type.byte_width());

                    match integer_value {
                        ConstantInteger::Zero => Self::zero(byte_width),
                        ConstantInteger::All => Self::with_byte(0xFFu8, byte_width),
                        ConstantInteger::One => {
                            let mut value = Self::zero(byte_width);
                            let index = if endianness == Endianness::Little {
                                0
                            } else {
                                byte_width.get() - 1
                            };
                            value.as_bytes_mut()[index] = 1u8;
                            value
                        }
                        _ => todo!("account for the endianness when calculating the values"),
                    }
                }
                Constant::Float(_) => panic!("cannot construct integer value from float constant"),
            },
            TypeKind::Float(float_type) => todo!("add support for float types {float_type:?}"),
        }
    }

    pub fn from_u8(byte: u8) -> Self {
        Self {
            length: unsafe {
                // Safety: Value is obviously not zero
                NonZeroUsize::new_unchecked(1)
            },
            bits: {
                let mut inlined = [0u8; POINTER_SIZE];
                inlined[0] = byte;
                Bits { inlined }
            },
        }
    }

    fn is_allocated(&self) -> bool {
        self.length.get() > POINTER_SIZE
    }

    /// The size of this value, in bytes.
    pub fn byte_width(&self) -> NonZeroUsize {
        self.length
    }

    /// Returns a slice containing this value's bytes.
    ///
    /// For a mutating version, see [`Value::as_bytes`].
    pub fn as_bytes(&self) -> &[u8] {
        if self.is_allocated() {
            unsafe {
                // Safety: Check above ensures that the pointer is valid
                std::slice::from_raw_parts(self.bits.allocated, self.length.get())
            }
        } else {
            let inlined = unsafe {
                // Safety: Check above ensures that the value was NOT allocated
                &self.bits.inlined
            };

            &inlined[0..self.length.get()]
        }
    }

    /// Returns a mutable slice containing this value's bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        if self.is_allocated() {
            unsafe {
                // Safety: Check above ensures that the pointer is valid
                std::slice::from_raw_parts_mut(self.bits.allocated, self.length.get())
            }
        } else {
            let inlined = unsafe {
                // Safety: Check above ensures that the value was NOT allocated
                &mut self.bits.inlined
            };

            &mut inlined[0..self.length.get()]
        }
    }

    /// Returns a `Box<[u8]>` containing this value's bytes.
    pub fn into_boxed_bytes(self) -> Box<[u8]> {
        let length = self.length.get();
        if self.is_allocated() {
            unsafe {
                // Safety: The check to see if a boxed slice was actually allocated occurs above
                Box::<[u8]>::from_raw(std::slice::from_raw_parts_mut(self.bits.allocated, length))
            }
        } else {
            let inlined = unsafe {
                // Safety: Value was inlined
                &self.bits.inlined[0..length]
            };

            Box::from(inlined)
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.as_bytes()).finish()
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl std::cmp::Eq for Value {}

impl From<Value> for Box<[u8]> {
    fn from(value: Value) -> Self {
        value.into_boxed_bytes()
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        if self.is_allocated() {
            let length = self.length.get();
            unsafe {
                // Safety: The check to see if a boxed slice was actually allocated occurs above
                Box::<[u8]>::from_raw(std::slice::from_raw_parts_mut(self.bits.allocated, length));
            }
        }
    }
}
