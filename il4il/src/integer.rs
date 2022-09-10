//! Module for manipulating integers in the IL4IL binary format.
//!
//! # Variable Length Encoding
//!
//! The encoding of variable-width integers is similar to the encoding used in UTF-8 codepoints. The lowest bits indicate the
//! number of bytes needed to contain the integer value.
//!
//! | Actual Value                          |Integer Length (bytes)|Integer Size (bits)|
//! |---------------------------------------|----------------------|-------------------|
//! | `XXXXXXX0`                            | `1`                  | `7`               |
//! | `XXXXXX01 XXXXXXXX`                   | `2`                  | `14`              |
//! | `XXXXX011 XXXXXXXX XXXXXXXX`          | `3`                  | `21`              |
//! | `XXXX0111 XXXXXXXX XXXXXXXX XXXXXXXX` | `4`                  | `28`              |
//!
//! For simplicity, the binary format currently only allows a maximum length of `4` for all integers.

use std::cmp::{Ord, PartialOrd};
use std::fmt::{Debug, Display, Formatter};
use std::num::{NonZeroU32, NonZeroU8};

/// Error type used when the indicated length of an integer is invalid.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("integers of byte length {length} are not supported by IL4IL")]
pub struct LengthError {
    length: u8,
}

/// Error type used when an attempt to store an integer value fails.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("integer too large to be encoded")]
pub struct EncodingError(());

const UNUSED_BITS: u32 = 0xF000_0000u32;

/// An unsigned integer that can be represented in 1, 2, 3, or 4 bytes.
///
/// For more details, see the documentation for the [this module].
///
/// [this module]: crate::integer
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VarU28(NonZeroU32); // Bit 0 permanently set, remaining bits contain value allowing null pointer optimization

impl VarU28 {
    /// Creates a new unsigned integer.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot fit in 28 bits.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        assert!(value & UNUSED_BITS == 0u32);
        Self(unsafe {
            // Safety: Bit 0 is always set
            NonZeroU32::new_unchecked(1u32 | (value << 1))
        })
    }

    /// The smallest value that can be encoded.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::MIN.get(), 0);
    /// ```
    pub const MIN: Self = Self::new(0);

    /// The largest value that is allowed to be encoded.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::MAX.get() >> VarU28::BITS, 0);
    /// ```
    pub const MAX: Self = Self::new(0x0FFF_FFFF);

    /// The number of bits that can encode a value.
    pub const BITS: u32 = 28u32;

    /// Gets the value of this integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::from_u16(99).get(), 99);
    /// ```
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get() >> 1
    }

    /// Creates a new unsigned integer, returning `None` if the value is too large to be represented in 28 bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::from_u32(99), Some(VarU28::from_u16(99)));
    /// assert_eq!(VarU28::from_u32(0x1000_0000), None);
    /// assert_eq!(VarU28::from_u32(u32::MAX), None);
    /// ```
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value & UNUSED_BITS == 0 {
            Some(Self::new(value))
        } else {
            None
        }
    }

    /// Creates an unsigned integer from an unsigned byte value.
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        Self::new(value as u32)
    }

    /// Creates an unsigned integer from an unsigned 16-bit integer.
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        Self::new(value as u32)
    }

    /// The maximum value that can be encoded in 1 byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert!(VarU28::MIN < VarU28::MAX_1);
    /// ```
    pub const MAX_1: Self = Self::from_u8(0x7F);

    /// The maximum value that can be encoded in 2 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert!(VarU28::MAX_2 < VarU28::MAX_3);
    /// ```
    pub const MAX_2: Self = Self::from_u16(0x3FFF);

    /// The maximum value that can be encoded in 3 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert!(VarU28::MAX_3 < VarU28::MAX_4);
    /// ```
    pub const MAX_3: Self = Self::new(0x001F_FFFF);

    /// The maximum value that can be encoded in 4 bytes.
    pub const MAX_4: Self = Self::MAX;

    /// Gets the number of bytes needed to contain this unsigned integer value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::from_u8(1).byte_length().get(), 1);
    /// assert_eq!(VarU28::MAX_1.byte_length().get(), 1);
    /// assert_eq!(VarU28::from_u8(u8::MAX).byte_length().get(), 2);
    /// assert_eq!(VarU28::from_u16(u16::MAX).byte_length().get(), 3);
    /// assert_eq!(VarU28::MAX.byte_length().get(), 4);
    /// ```
    #[must_use]
    pub fn byte_length(self) -> NonZeroU8 {
        unsafe {
            // Safety: All byte lengths are never zero
            NonZeroU8::new_unchecked(if self <= Self::MAX_1 {
                1u8
            } else if self <= Self::MAX_2 {
                2
            } else if self <= Self::MAX_3 {
                3
            } else if self <= Self::MAX_4 {
                4
            } else {
                unreachable!("value above maximum is not valid")
            })
        }
    }

    /// Reads a variable-length integer value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert!(matches!(VarU28::read_from([0b0110_1100u8].as_slice()), Ok(Ok(n)) if n.get() == 0b0011_0110));
    /// assert!(matches!(VarU28::read_from([1u8].as_slice()), Err(_)));
    /// ```
    pub fn read_from<R: std::io::Read>(mut source: R) -> std::io::Result<Result<Self, LengthError>> {
        let mut buffer = [0u8; 4];
        source.read_exact(&mut buffer[0..1])?;

        let trailing_one_count = buffer[0].trailing_ones();
        match trailing_one_count {
            0 => (),
            1 => source.read_exact(&mut buffer[1..2])?,
            2 => source.read_exact(&mut buffer[1..3])?,
            3 => source.read_exact(&mut buffer[1..4])?,
            byte_length => return Ok(Err(LengthError { length: byte_length as u8 })),
        }

        Ok(Ok(Self::new(u32::from_le_bytes(buffer) >> (trailing_one_count + 1))))
    }

    /// Writes a variable-length integer value.
    pub fn write_to<W: std::io::Write>(self, mut destination: W) -> std::io::Result<()> {
        let bytes = self.get();
        match self.byte_length().get() {
            1 => destination.write_all(&[(bytes as u8) << 1]),
            2 => {
                let mut buffer: [u8; 2] = ((bytes as u16) << 2).to_le_bytes();
                buffer[0] |= 0b01u8;
                destination.write_all(&buffer)
            }
            3 => {
                let mut buffer: [u8; 4] = (bytes << 3).to_le_bytes();
                buffer[0] |= 0b011u8;
                destination.write_all(&buffer[..3])
            }
            4 => {
                let mut buffer: [u8; 4] = (bytes << 4).to_le_bytes();
                buffer[0] |= 0b0111u8;
                destination.write_all(&buffer)
            }
            _ => unreachable!("unsupported byte length"),
        }
    }

    /// Allocates a [`Vec<u8>`] containing the representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarU28;
    /// assert_eq!(VarU28::from_u8(0b0101_0110).into_vec(), &[0b1010_1100u8]);
    /// assert_eq!(VarU28::from_u8(6).into_vec(), &[0b0000_1100u8]);
    /// assert_eq!(VarU28::from_u8(128).into_vec(), &[1u8, 2]);
    /// assert_eq!(VarU28::from_u8(255).into_vec(), &[0b11111101u8, 0b11]);
    /// assert_eq!(VarU28::MAX_1.into_vec(), &[0b1111_1110u8]);
    /// assert_eq!(VarU28::MAX_2.into_vec(), &[0b1111_1101u8, 0xFF]);
    /// assert_eq!(VarU28::MAX_3.into_vec(), &[0b1111_1011u8, 0xFF, 0xFF]);
    /// assert_eq!(VarU28::MAX_4.into_vec(), &[0xF7u8, 0xFF, 0xFF, 0xFF]);
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1);
        self.write_to(&mut bytes).unwrap();
        bytes
    }
}

impl Default for VarU28 {
    fn default() -> Self {
        Self::MIN
    }
}

impl From<u8> for VarU28 {
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}

impl From<u16> for VarU28 {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl TryFrom<u32> for VarU28 {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(EncodingError(()))
    }
}

impl TryFrom<usize> for VarU28 {
    type Error = EncodingError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(u32::try_from(value).map_err(|_| EncodingError(()))?)
    }
}

impl TryFrom<VarU28> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: VarU28) -> Result<usize, Self::Error> {
        usize::try_from(value.get())
    }
}

impl Debug for VarU28 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

impl Display for VarU28 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl std::ops::BitOr for VarU28 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for VarU28 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.get() & rhs.get())
    }
}

/// An signed integer that can be represented in 1, 2, 3, or 4 bytes.
///
/// For more details, see the documentation for the [this module].
///
/// [this module]: crate::integer
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct VarI28(NonZeroU32);

impl VarI28 {
    const SIGN_BIT: u32 = 0x0800_0000u32;

    /// Creates a new signed integer.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot fit in 28 bits.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        let mut bytes = value as u32;
        let leading_ones = bytes.leading_ones();
        assert!((leading_ones == 0 && bytes.leading_zeros() >= 4) || leading_ones > 4);
        if leading_ones != 0 {
            bytes &= !UNUSED_BITS;
        }
        Self(unsafe {
            // Safety: Bit 0 is always set
            NonZeroU32::new_unchecked(1u32 | ((bytes as u32) << 1))
        })
    }

    /// Creates a new signed integer, returning `None` if the value cannot fit in 28 bits.
    #[must_use]
    pub const fn from_i32(value: i32) -> Option<Self> {
        match VarU28::from_u32(value as u32) {
            Some(v) => Some(Self(v.0)),
            None => None,
        }
    }

    /// Creates a signed integer from an unsigned byte value.
    ///
    /// # Example
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::from_u8(21).get(), 21);
    /// assert_eq!(VarI28::from_u8(u8::MIN), VarI28::ZERO);
    /// assert_eq!(VarI28::from_u8(u8::MAX).get(), u8::MAX.into());
    /// ```
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        Self::new(value as i32)
    }

    /// Creates a signed integer from an unsigned 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::from_u16(42).get(), 42);
    /// assert_eq!(VarI28::from_u16(u16::MIN), VarI28::ZERO);
    /// assert_eq!(VarI28::from_u16(u16::MAX).get(), u16::MAX.into());
    /// ```
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        Self::new(value as i32)
    }

    /// Creates a signed integer from a signed byte value.
    ///
    /// # Example
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::from_i8(123).get(), 123);
    /// assert_eq!(VarI28::from_i8(-123).get(), -123);
    /// assert_eq!(VarI28::from_i8(i8::MIN).get(), i8::MIN.into());
    /// assert_eq!(VarI28::from_i8(i8::MAX).get(), i8::MAX.into());
    /// ```
    #[must_use]
    pub const fn from_i8(value: i8) -> Self {
        let b = value as u8;
        if b & 0x80u8 == 0 {
            Self::from_u8(b)
        } else {
            Self::new((0x0FFF_FF00u32 | (b as u32)) as i32)
        }
    }

    /// Creates a signed integer from a signed 16-bit integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::from_i16(456).get(), 456);
    /// assert_eq!(VarI28::from_i16(-456).get(), -456);
    /// assert_eq!(VarI28::from_i16(i16::MIN).get(), i16::MIN.into());
    /// assert_eq!(VarI28::from_i16(i16::MAX).get(), i16::MAX.into());
    /// ```
    #[must_use]
    pub const fn from_i16(value: i16) -> Self {
        let b = value as u16;
        if b & 0x8000u16 == 0 {
            Self::from_u16(b)
        } else {
            Self::new((0x0FFF_0000u32 | (b as u32)) as i32)
        }
    }

    /// Gets the zero value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::ZERO.get() == 0);
    /// ```
    pub const ZERO: Self = Self::from_u8(0);

    /// The maximum positive value that can be encoded in one byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::ZERO < VarI28::MAX_1);
    /// assert_eq!(VarI28::MAX_1.get(), 63);
    /// ```
    pub const MAX_1: Self = Self::from_u8(0b0011_1111u8);

    /// The minimum negative value that can be encoded in one byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MIN_1 < VarI28::ZERO);
    /// assert_eq!(VarI28::MIN_1.get(), -64);
    /// ```
    pub const MIN_1: Self = Self::from_i8(0b1100_0000u8 as i8);

    /// The maximum positive value that can be encoded in two bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MAX_1 < VarI28::MAX_2);
    /// assert_eq!(VarI28::MAX_2.get(), 8191);
    /// ```
    pub const MAX_2: Self = Self::from_u16(0b0001_1111_1111_1111u16);

    /// The minimum negative value that can be encoded in two bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MIN_2 < VarI28::MIN_1);
    /// assert_eq!(VarI28::MIN_2.get(), -8192);
    /// ```
    pub const MIN_2: Self = Self::from_i16(0b1110_0000_0000_0000u16 as i16);

    /// The maximum positive value that can be encoded in three bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MAX_2 < VarI28::MAX_3);
    /// assert_eq!(VarI28::MAX_3.get(), 1048575);
    /// ```
    pub const MAX_3: Self = Self::new(0x000F_FFFFi32);

    /// The minimum negative value that can be encoded in three bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MIN_3 < VarI28::MIN_2);
    /// assert_eq!(VarI28::MIN_3.get(), -1048576);
    /// ```
    pub const MIN_3: Self = Self::new(0x0FF0_0000u32 as i32);

    /// The maximum positive value that can be encoded in four bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MAX_3 < VarI28::MAX_4);
    /// assert_eq!(VarI28::MAX_4.get(), 134217727);
    /// ```
    pub const MAX_4: Self = Self::new(0x07FF_FFFFi32);

    /// The minimum negative value that can be encoded in four bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert!(VarI28::MIN_4 < VarI28::MIN_3);
    /// assert_eq!(VarI28::MIN_4.get(), -134217728);
    /// ```
    pub const MIN_4: Self = Self::new(0x0800_0000u32 as i32);

    /// Gets the value of this signed integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        let mut value = self.0.get() >> 1;
        if value & Self::SIGN_BIT != 0 {
            // Perform a sign extension
            value |= UNUSED_BITS;
        }
        value as i32
    }

    /// Gets the number of bytes needed to contain this signed integer value.
    #[must_use]
    pub fn byte_length(self) -> NonZeroU8 {
        // Safety: Sizes are guaranteed to not be zero.
        unsafe {
            NonZeroU8::new_unchecked(if self >= Self::MIN_1 && self <= Self::MAX_1 {
                1
            } else if self >= Self::MIN_2 && self <= Self::MAX_2 {
                2
            } else if self >= Self::MIN_3 && self <= Self::MAX_3 {
                3
            } else if self >= Self::MIN_4 && self <= Self::MAX_4 {
                4
            } else {
                unreachable!("value is not valid")
            })
        }
    }

    /// Writes a signed variable-length integer value.
    pub fn write_to<W: std::io::Write>(self, mut destination: W) -> std::io::Result<()> {
        let value = self.get();
        // Note that the sign bit is already correct
        match self.byte_length().get() {
            1 => destination.write_all(&[(value as u8) << 1]),
            2 => destination.write_all(&(((value as u16) << 2) | 1u16).to_le_bytes()),
            3 => destination.write_all(&(((value as u32) << 3) | 0b11u32).to_le_bytes()[0..3]),
            4 => destination.write_all(&(((value as u32) << 4) | 0b111u32).to_le_bytes()),
            _ => unreachable!(),
        }
    }

    /// Reads a variable-length signed integer value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::read_from([0].as_slice()).unwrap(), Ok(VarI28::ZERO));
    /// assert_eq!(VarI28::read_from([0b1100].as_slice()).unwrap().unwrap().get(), 6);
    /// assert_eq!(VarI28::read_from([0b1111_1100].as_slice()).unwrap().unwrap().get(), -2);
    /// assert_eq!(VarI28::read_from([0b0000_0001, 4].as_slice()).unwrap().unwrap().get(), 256);
    /// assert_eq!(VarI28::read_from([0b0000_0001, 0b1111_1000].as_slice()).unwrap().unwrap().get(), -512);
    /// assert_eq!(VarI28::read_from([0b0000_0011, 0, 8].as_slice()).unwrap().unwrap().get(), 65536);
    /// assert_eq!(VarI28::read_from([0b0001_0111, 0, 0, 8].as_slice()).unwrap().unwrap().get(), 8388609);
    /// ```
    pub fn read_from<R: std::io::Read>(mut source: R) -> std::io::Result<Result<Self, LengthError>> {
        let mut leading_byte = 0u8;
        source.read_exact(std::slice::from_mut(&mut leading_byte))?;

        Ok(match leading_byte.trailing_ones() {
            0 => {
                let mut value = leading_byte >> 1;
                if leading_byte & 0x80u8 != 0 {
                    value |= 0x80u8; // Sign extend
                }
                Ok(Self::from_i8(value as i8))
            }
            1 => {
                let mut buffer = [leading_byte, 0];
                source.read_exact(&mut buffer[1..])?;
                let bytes = u16::from_le_bytes(buffer);
                let mut value = bytes >> 2;
                if bytes & 0x8000u16 != 0 {
                    value |= 0xC000u16; // Sign extend
                }
                Ok(Self::from_i16(value as i16))
            }
            2 => {
                let mut buffer = [leading_byte, 0, 0, 0];
                source.read_exact(&mut buffer[1..3])?;
                let bytes = u32::from_le_bytes(buffer);
                let mut value = bytes >> 3;
                if bytes & 0x0080_0000u32 != 0 {
                    value |= 0xFFE0_0000u32; // Sign extend
                }
                Ok(Self::new(value as i32))
            }
            3 => {
                let mut buffer = [leading_byte, 0, 0, 0];
                source.read_exact(&mut buffer[1..])?;
                let bytes = u32::from_le_bytes(buffer);
                let mut value = bytes >> 4;
                if bytes & 0x8000_0000u32 != 0 {
                    value |= 0xF000_0000u32; // Sign extend
                }
                Ok(Self::new(value as i32))
            }
            byte_length => Err(LengthError { length: byte_length as u8 }),
        })
    }

    /// Returns a `Vec` containing the representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::integer::VarI28;
    /// assert_eq!(VarI28::ZERO.into_vec(), &[0]);
    /// assert_eq!(VarI28::from_i8(5).into_vec(), &[0b0000_1010]);
    /// assert_eq!(VarI28::from_i8(-5).into_vec(), &[0b1111_0110]);
    /// assert_eq!(VarI28::from_i8(64).into_vec(), &[1, 1]);
    /// assert_eq!(VarI28::from_i8(-64).into_vec(), &[0b1000_0000]);
    /// assert_eq!(VarI28::MAX_1.into_vec(), &[0b0111_1110]);
    /// assert_eq!(VarI28::MIN_1.into_vec(), &[0x80]);
    /// assert_eq!(VarI28::MAX_2.into_vec(), &[0b1111_1101, 0x7F]);
    /// assert_eq!(VarI28::MIN_2.into_vec(), &[0b0000_0001, 0x80]);
    /// assert_eq!(VarI28::MAX_3.into_vec(), &[0b1111_1011, 0xFF, 0x7F]);
    /// assert_eq!(VarI28::MIN_3.into_vec(), &[0b0000_0011, 0, 0x80]);
    /// assert_eq!(VarI28::MAX_4.into_vec(), &[0b1111_0111, 0xFF, 0xFF, 0x7F]);
    /// assert_eq!(VarI28::MIN_4.into_vec(), &[0b0000_0111, 0, 0, 0x80]);
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1);
        self.write_to(&mut bytes).unwrap();
        bytes
    }
}

impl Default for VarI28 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl PartialOrd for VarI28 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl Ord for VarI28 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl Debug for VarI28 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

impl Display for VarI28 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl TryFrom<VarI28> for VarU28 {
    type Error = std::num::TryFromIntError;

    fn try_from(value: VarI28) -> Result<Self, Self::Error> {
        u32::try_from(value.get()).map(VarU28::new)
    }
}

#[cfg(test)]
mod tests {
    use crate::integer::{VarI28, VarU28};
    use crate::propcheck;

    impl propcheck::Arb for VarU28 {
        type Shrinker = std::iter::Empty<Self>;

        fn arbitrary<R: propcheck::Rng + ?Sized>(gen: &mut propcheck::Gen<'_, R>) -> Self {
            Self::new(gen.source().gen_range(0..=Self::MAX.get()))
        }

        fn shrink(&self) -> Self::Shrinker {
            std::iter::empty()
        }
    }

    impl propcheck::Arb for VarI28 {
        type Shrinker = std::iter::Empty<Self>;

        fn arbitrary<R: propcheck::Rng + ?Sized>(gen: &mut propcheck::Gen<'_, R>) -> Self {
            Self::new(gen.source().gen_range(Self::MIN_4.get()..=Self::MAX_4.get()))
        }

        fn shrink(&self) -> Self::Shrinker {
            std::iter::empty()
        }
    }

    propcheck::property! {
        fn u28_bitwise_or_matches(left: VarU28, right: VarU28) {
            propcheck::assertion_eq!((left | right).get(), left.get() | right.get())
        }
    }

    propcheck::property! {
        fn u28_bitwise_and_matches(left: VarU28, right: VarU28) {
            propcheck::assertion_eq!((left & right).get(), left.get() & right.get())
        }
    }

    propcheck::property! {
        fn written_u28_can_be_parsed(value: VarU28) {
            let bytes = value.into_vec();
            propcheck::assertion_eq!(VarU28::read_from(bytes.as_slice()).unwrap(), Ok(value))
        }
    }

    propcheck::property! {
        fn written_i28_can_be_parsed(value: VarI28) {
            let bytes = value.into_vec();
            propcheck::assertion_eq!(VarI28::read_from(bytes.as_slice()).unwrap(), Ok(value))
        }
    }
}
