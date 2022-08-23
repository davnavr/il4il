//! Contains types that describe the environment that an IL4IL module is loaded in.

/// Indicates the size of pointer addresses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct AddressSize(std::num::NonZeroU16);

impl AddressSize {
    pub const fn with_byte_size(size: std::num::NonZeroU16) -> Self {
        Self(size)
    }

    pub const fn byte_size(self) -> std::num::NonZeroU16 {
        self.0
    }

    pub const fn bit_size(self) -> std::num::NonZeroU32 {
        unsafe {
            // Safety: Address size is guaranteed to never be zero.
            std::num::NonZeroU32::new_unchecked((self.0.get() as u32) * 8)
        }
    }

    pub const NATIVE: Self = unsafe {
        // Safety: Size of pointers in Rust is assumed to never be zero.
        Self::with_byte_size(std::num::NonZeroU16::new_unchecked(std::mem::size_of::<usize>() as u16))
    };
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Context {
    /// Specifies the sizes of pointer addresses for all modules in this context.
    ///
    /// This affects certain aspects of loading, such as type size calculation.
    pub address_size: AddressSize,
}
