//! Module for manipulation of values encoded in IL4IL instructions.

use crate::integer::VarI28;

#[derive(Clone, Debug, thiserror::Error)]
#[error("{tag} is not a valid constant value tag")]
pub struct InvalidTagError {
    tag: i32,
}

macro_rules! constant_tag {
    {$($name:ident = $value:literal,)*} => {
        /// Indicates the kind of value contained in a [`Constant`]. Each tag value corresponds to negative values in the
        /// [variable-length signed integer encoding](VarI28).
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(i8)]
        pub enum ConstantTag {
            $($name = $value,)*
        }

        impl ConstantTag {
            pub const fn from_i32(tag: i32) -> Option<Self> {
                match tag {
                    $($value => Some(Self::$name),)*
                    _ => None,
                }
            }
        }

        impl TryFrom<VarI28> for ConstantTag {
            type Error = InvalidTagError;

            fn try_from(tag: VarI28) -> Result<Self, Self::Error> {
                let value = tag.get();
                Self::from_i32(value).ok_or(InvalidTagError { tag: value })
            }
        }

        impl From<ConstantTag> for VarI28 {
            fn from(tag: ConstantTag) -> VarI28 {
                VarI28::from_i8(match tag {
                    $(ConstantTag::$name => $value,)*
                })
            }
        }
    };
}

constant_tag! {
    IntegerZero = -1,
    IntegerOne = -2,
    IntegerAll = -3,
    IntegerSignedMaximum = -4,
    IntegerSignedMinimum = -5,
    IntegerInline8 = -6,
    IntegerInline16 = -7,
    IntegerInline32 = -8,
    IntegerInline64 = -9,
    IntegerInline128 = -10,
    Float16 = -21,
    Float32 = -22,
    Float64 = -23,
    Float128 = -24,
}

/// A constant integer value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstantInteger {
    /// An integer value with no bits set, also the `false` boolean value.
    Zero,
    /// An integer value with only the lowest bit set, also the `true` boolean value.
    One,
    /// An integer value with all bits set.
    All,
    /// An integer value corresponding to the most positive value if it was interpreted as a signed twos-complement integer.
    SignedMaximum,
    /// An integer value corresponding to the most negative value if it was interpreted as a signed twos-complement integer.
    SignedMinimum,
    /// A byte value.
    Byte(u8),
    /// A 16-bit integer stored in little-endian order.
    I16([u8; 2]),
    /// A 32-bit integer stored in little-endian order.
    I32([u8; 4]),
    /// A 64-bit integer stored in little-endian order.
    I64([u8; 8]),
    /// A 128-bit integer stored in little-endian order.
    I128([u8; 16]),
}

impl ConstantInteger {
    pub fn tag(&self) -> ConstantTag {
        match self {
            Self::Zero => ConstantTag::IntegerZero,
            Self::One => ConstantTag::IntegerOne,
            Self::All => ConstantTag::IntegerAll,
            Self::SignedMaximum => ConstantTag::IntegerSignedMaximum,
            Self::SignedMinimum => ConstantTag::IntegerSignedMinimum,
            Self::Byte(_) => ConstantTag::IntegerInline8,
            Self::I16(_) => ConstantTag::IntegerInline16,
            Self::I32(_) => ConstantTag::IntegerInline32,
            Self::I64(_) => ConstantTag::IntegerInline64,
            Self::I128(_) => ConstantTag::IntegerInline128,
        }
    }
}

/// A constant floating-point value, stored in little-endian order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstantFloat {
    Half([u8; 2]),
    Single([u8; 4]),
    Double([u8; 8]),
    Quadruple([u8; 16]),
}

impl ConstantFloat {
    pub fn bit_width(&self) -> std::num::NonZeroU16 {
        unsafe {
            // Safety: Values below are not zero
            std::num::NonZeroU16::new_unchecked(match self {
                Self::Half(_) => 16,
                Self::Single(_) => 32,
                Self::Double(_) => 64,
                Self::Quadruple(_) => 128,
            })
        }
    }

    pub fn tag(&self) -> ConstantTag {
        match self {
            Self::Half(_) => ConstantTag::Float16,
            Self::Single(_) => ConstantTag::Float32,
            Self::Double(_) => ConstantTag::Float64,
            Self::Quadruple(_) => ConstantTag::Float128,
        }
    }
}

impl From<f32> for ConstantFloat {
    fn from(single: f32) -> Self {
        Self::Single(single.to_le_bytes())
    }
}

impl From<f64> for ConstantFloat {
    fn from(double: f64) -> Self {
        Self::Double(double.to_le_bytes())
    }
}

/// A constant value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Constant {
    Integer(ConstantInteger),
    Float(ConstantFloat),
}

impl Constant {
    pub fn tag(&self) -> ConstantTag {
        match self {
            Self::Integer(integer) => integer.tag(),
            Self::Float(float) => float.tag(),
        }
    }
}

/// A value used as an immediate argument for some IL4IL instructions.
///
/// In many cases, the type of the argument is inferred, though some instructions may explicitly require a type for a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    Constant(Constant),
    ///// An index to a register, encoded as a [variable-length signed integer](VarI28).
    //Register(crate::index::Register),
}

impl From<ConstantInteger> for Value {
    fn from(i: ConstantInteger) -> Self {
        Self::Constant(Constant::Integer(i))
    }
}

impl From<ConstantFloat> for Value {
    fn from(f: ConstantFloat) -> Self {
        Self::Constant(Constant::Float(f))
    }
}

macro_rules! integer_to_constant_conversions {
    {$($integer:ty => $case_name:ident,)*} => {
        $(
            impl From<$integer> for ConstantInteger {
                fn from(value: $integer) -> Self {
                    Self::$case_name(value.to_le_bytes())
                }
            }

            impl From<$integer> for Constant {
                fn from(value: $integer) -> Self {
                    Self::Integer(ConstantInteger::from(value))
                }
            }

            impl From<$integer> for Value {
                fn from(value: $integer) -> Self {
                    Self::Constant(Constant::from(value))
                }
            }
        )*
    };
}

integer_to_constant_conversions! {
    i16 => I16,
    u16 => I16,
    i32 => I32,
    u32 => I32,
    i64 => I64,
    u64 => I64,
    i128 => I128,
    u128 => I128,
}
