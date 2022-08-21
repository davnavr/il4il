//! Module for manipulation of values encoded in IL4IL instructions.

use crate::integer::{VarI28, VarU28};

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
    Integer8 = -6,
    Integer16 = -7,
    Integer32 = -8,
    Integer64 = -9,
    Integer128 = -10,
}

/// A constant integer value.
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// A constant floating-point value, stored in little-endian order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstantFloat {
    Single([u8; 4]),
    Double([u8; 8]),
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constant {
    Integer(ConstantInteger),
    Float(ConstantFloat),
}

/// A value used as an immediate argument for some IL4IL instructions.
///
/// In many cases, the type of the argument is inferred, though some instructions may explicitly require a type for a value.
#[derive(Clone, Debug, Eq, PartialEq)]
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
