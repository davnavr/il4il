//! Provides a model of the IL4IL type system.

use crate::integer::{VarI28, VarU28};
use std::fmt::{Debug, Display, Formatter, Write};
use std::num::{NonZeroU16, NonZeroU8};

macro_rules! type_tag {
    ($($(#[$meta:meta])* $name:ident = $written_value:literal => $interpreted_value:literal,)*) => {
        /// Tag that reprsents a [`Type`]. Note that all type tags correspond to negative numbers in the IL4IL
        /// [variable-length signed integer encoding](#VarI28), allowing positive values to represent indices into a
        /// module's type section.
        ///
        /// Common types (bool, s32, u32, f32, s64, f64, etc.) are represented as special cases for efficiency.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum TypeTag {
            $($(#[$meta])* $name = $written_value,)*
        }

        impl TypeTag {
            pub const ALL: &'static [Self] = &[$(Self::$name,)*];

            pub const fn new(tag: u8) -> Option<Self> {
                match tag {
                    $(_ if tag == $written_value => Some(Self::$name),)*
                    _ => None,
                }
            }

            pub const fn from_i28(tag: VarI28) -> Option<Self> {
                match tag.get() {
                    $(value if value == $interpreted_value => Some(Self::$name),)*
                    _ => None
                }
            }

            pub const fn into_i28(self) -> VarI28 {
                VarI28::from_i8(match self {
                    $(Self::$name => $interpreted_value,)*
                })
            }
        }
    };
}

type_tag! {
    Bool = 0xFE => -1,
    U8 = 0xFC => -2,
    S8 = 0xFA => -3,
    U16 = 0xF8 => -4,
    S16 = 0xF6 => -5,
    U32 = 0xF4 => -6,
    S32 = 0xF2 => -7,
    U64 = 0xF0 => -8,
    S64 = 0xEE => -9,
    U128 = 0xEC => -10,
    S128 = 0xEA => -11,
    U256 = 0xE8 => -12,
    S256 = 0xE6 => -13,
    UAddr = 0xE4 => -14,
    SAddr = 0xE2 => -15,
    /// An unsigned integer type with an arbitrary size.
    UInt = 0xE0 => -16,
    /// A signed integer type with an arbitrary size.
    SInt = 0xDE => -17,
    F16 = 0xDC => -18,
    F32 = 0xDA => -19,
    F64 = 0xD8 => -20,
    F128 = 0xD6 => -21,
    F256 = 0xD4 => -22,
}

impl From<TypeTag> for u8 {
    fn from(tag: TypeTag) -> u8 {
        tag as u8
    }
}

impl From<TypeTag> for VarI28 {
    fn from(tag: TypeTag) -> VarI28 {
        tag.into_i28()
    }
}

/// The error type used when a negative variable-length integer does not correspond to the encoding of any type.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0} is not a valid type tag")]
pub struct InvalidTagError(VarI28);

impl TryFrom<VarI28> for TypeTag {
    type Error = InvalidTagError;

    fn try_from(value: VarI28) -> Result<Self, Self::Error> {
        Self::from_i28(value).ok_or(InvalidTagError(value))
    }
}

/// The error type used for invalid integer bit widths.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0} is not a supported integer bit width")]
pub struct InvalidBitWidthError(VarU28);

/// Represents the integer sizes supported by IL4IL.
///
/// Note that an integer size of 1 is not allowed, and is instead represented by [`SizedInteger::BOOL`].
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// Creates an integer size from a bit width that is greater than 1.
    ///
    /// # Safety
    ///
    /// The bit width must be greater than or equal to two.
    pub const unsafe fn new_unchecked(bit_width: u8) -> Self {
        Self(unsafe {
            // Safety: If bit width is not 1 or 0, size is never zero
            NonZeroU8::new_unchecked(bit_width - 1u8)
        })
    }

    /// Creates an integer size from a bit width. Returns `None` if the bit width is less than two.
    pub const fn new(bit_width: u8) -> Option<Self> {
        if bit_width >= 2u8 {
            Some(unsafe {
                // Safety: Check above ensures size is at least two
                Self::new_unchecked(bit_width)
            })
        } else {
            None
        }
    }

    pub fn from_u28(bit_width: VarU28) -> Result<Self, InvalidBitWidthError> {
        let bits = u8::try_from(bit_width.get()).map_err(|_| InvalidBitWidthError(bit_width))?;
        Self::new(bits).ok_or(InvalidBitWidthError(bit_width))
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
/// This includes the 1-bit `bool` type, and the signed (`s2`..`s256`) and unsigned (`u2`..`u256`) integer types.
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct SizedInteger(NonZeroU16); // Bits 8 to 15 store the size, bit 1 stores the sign, bit 0 always set.

impl SizedInteger {
    /// The 1-bit `bool` type, where a value of `1` represents true, and `0` represents false.
    pub const BOOL: Self = Self(unsafe { NonZeroU16::new_unchecked(1) });

    /// The unsigned byte type, an 8-bit integer.
    pub const U8: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I8);

    /// The signed byte type, an 8-bit integer.
    pub const S8: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I8);

    /// An unsigned 16-bit integer type.
    pub const U16: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I16);

    /// A signed 16-bit integer type.
    pub const S16: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I16);

    /// An unsigned 32-bit integer type.
    pub const U32: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I32);

    /// A signed 32-bit integer type, commonly used as the typical `int` type in many programming languages.
    pub const S32: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I32);

    /// An unsigned 64-bit integer type.
    pub const U64: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I64);

    /// A signed 64-bit integer type.
    pub const S64: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I64);

    /// An unsigned 128-bit integer type.
    pub const U128: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I128);

    /// A signed 128-bit integer type.
    pub const S128: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I128);

    /// An unsigned 256-bit integer type.
    pub const U256: Self = Self::new(IntegerSign::UNSIGNED, IntegerSize::I256);

    /// A signed 256-bit integer type.
    pub const S256: Self = Self::new(IntegerSign::SIGNED, IntegerSize::I256);

    /// Creates an integer type of a fixed size.
    pub const fn new(sign: IntegerSign, size: IntegerSize) -> Self {
        Self(unsafe {
            // Safety: Bit 0 is always set, so value is not zero
            NonZeroU16::new_unchecked(u16::from_le_bytes([sign.0.get(), size.0.get()]))
        })
    }

    /// Attempts to retrieve the size of this integer, returning `None` if this integer is a `bool`.
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
    /// assert_eq!(SizedInteger::BOOL.bit_width().get(), 1);
    /// assert_eq!(SizedInteger::new(IntegerSign::UNSIGNED, IntegerSize::new(24).unwrap()).bit_width().get(), 24);
    /// ```
    pub const fn bit_width(self) -> NonZeroU16 {
        if let Some(size) = self.size() {
            size.bit_width()
        } else {
            unsafe { NonZeroU16::new_unchecked(1) }
        }
    }

    /// Indicates whether this integer type is the `bool` type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::{IntegerSign, IntegerSize, SizedInteger};
    /// assert!(SizedInteger::BOOL.is_boolean());
    /// assert!(!SizedInteger::new(IntegerSign::UNSIGNED, IntegerSize::new(12).unwrap()).is_boolean());
    /// ```
    pub const fn is_boolean(self) -> bool {
        self.0.get() == 1
    }

    /// Gets whether the integer type is signed or unsigned, or `None` if the integer type is a `bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::{IntegerSign, IntegerSize, SizedInteger};
    /// assert_eq!(SizedInteger::BOOL.sign(), None);
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
    /// # use il4il::type_system::Float;
    /// assert_eq!(Float::F16.bit_width().get(), 16);
    /// assert_eq!(Float::F16.byte_width().get(), 2);
    /// ```
    pub const F16: Self = Self(unsafe { NonZeroU8::new_unchecked(1) });

    /// The 32-bit floating-point type, sometimes called `single`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::Float;
    /// assert_eq!(Float::F32.bit_width().get(), 32);
    /// assert_eq!(Float::F32.byte_width().get(), 4);
    /// ```
    pub const F32: Self = Self(unsafe { NonZeroU8::new_unchecked(2) });

    /// The 64-bit floating-point type, sometimes called `double`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::Float;
    /// assert_eq!(Float::F64.bit_width().get(), 64);
    /// assert_eq!(Float::F64.byte_width().get(), 8);
    /// ```
    pub const F64: Self = Self(unsafe { NonZeroU8::new_unchecked(3) });

    /// The 128-bit floating-point type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::Float;
    /// assert_eq!(Float::F128.bit_width().get(), 128);
    /// assert_eq!(Float::F128.byte_width().get(), 16);
    /// ```
    pub const F128: Self = Self(unsafe { NonZeroU8::new_unchecked(4) });

    /// The 256-bit floating-point type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::type_system::Float;
    /// assert_eq!(Float::F256.bit_width().get(), 256);
    /// assert_eq!(Float::F256.byte_width().get(), 32);
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

impl Display for Float {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.bit_width())
    }
}

/// Represents the set of all types representable in IL4IL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Type {
    Integer(Integer),
    Float(Float),
    //Array
    //Vector
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => Display::fmt(i, f),
            Self::Float(r) => Display::fmt(r, f),
        }
    }
}

impl From<SizedInteger> for Type {
    fn from(ty: SizedInteger) -> Self {
        Self::Integer(Integer::Sized(ty))
    }
}

/// An IL4IL type, or an index to an IL4IL type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reference {
    Inline(Type),
    Index(crate::index::Type),
}

impl From<Type> for Reference {
    fn from(ty: Type) -> Self {
        Self::Inline(ty)
    }
}

impl From<crate::index::Type> for Reference {
    fn from(index: crate::index::Type) -> Self {
        Self::Index(index)
    }
}

impl From<SizedInteger> for Reference {
    fn from(i: SizedInteger) -> Self {
        Self::Inline(Type::from(i))
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline(ty) => Display::fmt(ty, f),
            Self::Index(i) => write!(f, "#{}", usize::from(*i)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_tags_are_all_negative_variable_length_integers() {
        let results = TypeTag::ALL
            .iter()
            .copied()
            .map(|tag| VarI28::read_from([u8::from(tag)].as_slice()))
            .collect::<Vec<_>>();

        if !results.iter().all(|result| matches!(result, Ok(Ok(value)) if value.get() < 0)) {
            panic!("{:?}", results);
        }
    }

    #[test]
    fn type_tags_have_correct_values() {
        let expected = TypeTag::ALL
            .iter()
            .copied()
            .map(|tag| Some(tag.into_i28()))
            .collect::<Vec<Option<VarI28>>>();

        let actual = TypeTag::ALL
            .iter()
            .copied()
            .map(|tag| VarI28::read_from([u8::from(tag)].as_slice()).ok().and_then(Result::ok))
            .collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }
}
