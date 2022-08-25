//! Contains types that describe the environment that an IL4IL module is loaded in.

use il4il::type_system::IntegerSize;

/// Indicates the size of pointer addresses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct AddressSize(IntegerSize);

impl AddressSize {
    pub const fn from_integer_size(size: IntegerSize) -> Self {
        Self(size)
    }

    pub const fn size(self) -> IntegerSize {
        self.0
    }

    pub const fn bit_width(self) -> std::num::NonZeroU16 {
        self.0.bit_width()
    }

    /// The size of pointer addresses in 32-bit architectures.
    pub const BITS_32: Self = Self::from_integer_size(IntegerSize::I32);

    /// The size of pointer addresses in 64-bit architectures.
    pub const BITS_64: Self = Self::from_integer_size(IntegerSize::I64);

    /// The size of pointer addresses for the current CPU architecture.
    pub const NATIVE: Self = Self::from_integer_size(unsafe {
        // Safety: Rust address size is assumed to be greater than 1
        IntegerSize::new_unchecked((std::mem::size_of::<usize>() as u8) * 8u8)
    });
}

impl From<AddressSize> for IntegerSize {
    fn from(size: AddressSize) -> Self {
        size.0
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Context {
    /// Specifies the sizes of pointer addresses for all modules in this context.
    ///
    /// This affects certain aspects of loading, such as type size calculation.
    pub address_size: AddressSize,
}
