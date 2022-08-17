//! Provides a model of the IL4IL type system.

use std::fmt::{Debug, Display, Formatter, Write};
use std::num::{NonZeroU16, NonZeroU8};

/// Represents the integer sizes supported by IL4IL.
///
/// Note that an integer size of 1 is not allowed, and is instead represented by [`SizedInteger::BOOLEAN`].
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct IntegerSize(NonZeroU8);

impl IntegerSize {
    /// Represents an integer with a size of two bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::MIN.bit_width().get(), 2);
    /// ```
    pub const MIN: Self = Self(unsafe { NonZeroU8::new_unchecked(1u8) });

    /// The size of a byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I8.bit_width().get(), 8);
    /// ```
    pub const I8: Self = Self(unsafe { NonZeroU8::new_unchecked(7u8) });

    /// The size of a 16-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I16.bit_width().get(), 16);
    /// ```
    pub const I16: Self = Self(unsafe { NonZeroU8::new_unchecked(15u8) });

    /// The size of a 32-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I32.bit_width().get(), 32);
    /// ```
    pub const I32: Self = Self(unsafe { NonZeroU8::new_unchecked(31u8) });

    /// The size of a 64-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I64.bit_width().get(), 64);
    /// ```
    pub const I64: Self = Self(unsafe { NonZeroU8::new_unchecked(63u8) });

    /// The size of a 128-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I128.bit_width().get(), 128);
    /// ```
    pub const I128: Self = Self(unsafe { NonZeroU8::new_unchecked(127u8) });

    /// The size of a 256-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::IntegerSize;
    /// assert_eq!(IntegerSize::I256.bit_width().get(), 256);
    /// ```
    pub const I256: Self = Self(unsafe { NonZeroU8::new_unchecked(255u8) });

    /// The maximum allowed bit width of fixed width integers in IL4IL.
    pub const MAX: Self = Self::I256;

    /// Creates an integer size from a bit width.
    pub const fn new(bit_width: u8) -> Option<Self> {
        if bit_width >= 2u8 {
            Some(Self(unsafe {
                // Safety: Size is guaranteed to never be zero
                NonZeroU8::new_unchecked(bit_width - 1u8)
            }))
        } else {
            None
        }
    }

    /// Gets the number of bits needed to contain an integer of this size.
    pub const fn bit_width(self) -> NonZeroU16 {
        unsafe {
            // Safety: width is guaranteed to never be zero
            NonZeroU16::new_unchecked((self.0.get() as u16) + 1u16)
        }
    }
}

impl Debug for IntegerSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("IntegerSize").field(&self.bit_width()).finish()
    }
}

impl Display for IntegerSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.bit_width(), f)
    }
}

/// Indicates whether an integer type is signed or unsigned.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct IntegerSign(NonZeroU8);

impl IntegerSign {
    /// Indicates that the integer type is signed, with the most significant bit of a given value indicating the sign of the integer.
    pub const SIGNED: Self = Self(unsafe { NonZeroU8::new_unchecked(2) });

    /// Indicates that the integer type is unsigned, all values are zero or positive.
    pub const UNSIGNED: Self = Self(unsafe { NonZeroU8::new_unchecked(1) });

    pub const fn is_signed(self) -> bool {
        self.0.get() == 2
    }
}

impl Debug for IntegerSign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.is_signed() {
            f.write_str("UN")?;
        }
        f.write_str("SIGNED")
    }
}

impl Display for IntegerSign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(if self.is_signed() { 's' } else { 'u' })
    }
}

/// Represents the set of integer types with a fixed bit width supported by IL4IL.
///
/// This includes the 1-bit `boolean` type, and the signed (`s2`..`s256`) and unsigned (`u2`..`u256`) integer types.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct SizedInteger(NonZeroU16); // Bits 8 to 15 store the size, bit 1 stores the sign, bit 0 always set.

impl SizedInteger {
    /// The 1-bit `boolean` type, where a value of `1` represents true, and `0` represents false.
    pub const BOOLEAN: Self = Self(unsafe { NonZeroU16::new_unchecked(1) });

    /// Creates an integer type of a fixed size.
    pub const fn new(sign: IntegerSign, size: IntegerSize) -> Self {
        Self(unsafe {
            // Safety: Bit 0 is always set, so value is not zero
            NonZeroU16::new_unchecked(u16::from_le_bytes([sign.0.get(), size.0.get()]))
        })
    }

    /// Attempts to retrieve the size of this integer, returning `None` if this integer is a `boolean`.
    pub const fn size(self) -> Option<IntegerSize> {
        if let Some(size_bits) = NonZeroU8::new((self.0.get() >> 8) as u8) {
            Some(IntegerSize(size_bits))
        } else {
            None
        }
    }

    /// Gets the size of this integer type, in bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::{IntegerSign, IntegerSize, SizedInteger};
    /// assert_eq!(SizedInteger::BOOLEAN.bit_width().get(), 1);
    /// assert_eq!(SizedInteger::new(IntegerSign::UNSIGNED, IntegerSize::new(24).unwrap()).bit_width().get(), 24);
    /// ```
    pub const fn bit_width(self) -> NonZeroU16 {
        if let Some(size) = self.size() {
            size.bit_width()
        } else {
            unsafe { NonZeroU16::new_unchecked(1) }
        }
    }

    /// Indicates whether this integer type is the `boolean` type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::{IntegerSign, IntegerSize, SizedInteger};
    /// assert!(SizedInteger::BOOLEAN.is_boolean());
    /// assert!(!SizedInteger::new(IntegerSign::UNSIGNED, IntegerSize::new(12).unwrap()).is_boolean());
    /// ```
    pub const fn is_boolean(self) -> bool {
        self.0.get() == 1
    }

    /// Gets whether the integer type is signed or unsigned, or `None` if the integer type is a `boolean`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::{IntegerSign, IntegerSize, SizedInteger};
    /// assert_eq!(SizedInteger::BOOLEAN.sign(), None);
    /// assert_eq!(SizedInteger::new(IntegerSign::SIGNED, IntegerSize::new(40).unwrap()).sign(), Some(IntegerSign::SIGNED));
    /// ```
    pub const fn sign(self) -> Option<IntegerSign> {
        if self.is_boolean() {
            None
        } else {
            Some(IntegerSign(unsafe { NonZeroU8::new_unchecked(self.0.get() as u8) }))
        }
    }
}

impl Debug for SizedInteger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_boolean() {
            f.write_str("Boolean")
        } else {
            f.debug_struct("Integer")
                .field("size", &self.size().unwrap())
                .field("sign", &self.sign().unwrap())
                .finish()
        }
    }
}

impl Display for SizedInteger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_boolean() {
            f.write_str("boolean")
        } else {
            write!(f, "{}{}", self.sign().unwrap(), self.size().unwrap())
        }
    }
}

/// Represents the set of all integer types.
///
/// The values of integers in IL4IL are in two's complement representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Integer {
    /// An integer type with a fixed bit width.
    Sized(SizedInteger),
    /// An integer with the same bit width as a raw pointer's address.
    Address(IntegerSign),
}

impl From<SizedInteger> for Integer {
    fn from(sized_type: SizedInteger) -> Self {
        Self::Sized(sized_type)
    }
}

impl Display for Integer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sized(sized) => Display::fmt(sized, f),
            Self::Address(sign) => write!(f, "{}addr", sign),
        }
    }
}

/// Represents the floating-point types supported by IL4IL.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Float(NonZeroU8);

impl Float {
    /// The 16-bit floating-point type, sometimes called `half`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::FloatSize;
    /// assert_eq!(FloatSize::F16.bit_width().get(), 16);
    /// assert_eq!(FloatSize::F16.byte_width().get(), 2);
    /// ```
    pub const F16: Self = Self(unsafe { NonZeroU8::new_unchecked(1) });

    /// The 32-bit floating-point type, sometimes called `single`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::FloatSize;
    /// assert_eq!(FloatSize::F32.bit_width().get(), 32);
    /// assert_eq!(FloatSize::F32.byte_width().get(), 4);
    /// ```
    pub const F32: Self = Self(unsafe { NonZeroU8::new_unchecked(2) });

    /// The 64-bit floating-point type, sometimes called `double`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::FloatSize;
    /// assert_eq!(FloatSize::F64.bit_width().get(), 64);
    /// assert_eq!(FloatSize::F64.byte_width().get(), 8);
    /// ```
    pub const F64: Self = Self(unsafe { NonZeroU8::new_unchecked(3) });

    /// The 128-bit floating-point type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::FloatSize;
    /// assert_eq!(FloatSize::F128.bit_width().get(), 128);
    /// assert_eq!(FloatSize::F128.byte_width().get(), 16);
    /// ```
    pub const F128: Self = Self(unsafe { NonZeroU8::new_unchecked(4) });

    /// The 256-bit floating-point type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::FloatSize;
    /// assert_eq!(FloatSize::F256.bit_width().get(), 256);
    /// assert_eq!(FloatSize::F256.byte_width().get(), 32);
    /// ```
    pub const F256: Self = Self(unsafe { NonZeroU8::new_unchecked(5) });

    /// Gets the number of bits needed to contain floating-point values of this type.
    pub const fn bit_width(self) -> NonZeroU16 {
        unsafe {
            // Safety: Shifting won't result in a zero value here
            NonZeroU16::new_unchecked(2u16 << (self.0.get() + 2))
        }
    }

    /// Gets the number of bytes needed to contain floating-point values of this type.
    pub const fn byte_width(self) -> NonZeroU8 {
        unsafe {
            // Safety: Shifting won't result in a zero value here
            NonZeroU8::new_unchecked(2u8 << (self.0.get() - 1))
        }
    }
}

//pub enum Signature
