//! Types to model version numbers in IL4IL modules.

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// Represents an IL4IL binary format version number.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Format {
    /// The major version number, incremented when changes are made to the format that are incompatible with previous versions.
    pub major: u8,
    /// The minor version number, incremented when changes are made to the format that are compatible with previous versions.
    pub minor: u8,
}

impl Format {
    /// Creates a new format version with specified the major and minor version numbers.
    #[must_use]
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
}

impl Ord for Format {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => self.minor.cmp(&other.minor),
            ordering => ordering,
        }
    }
}

impl PartialOrd for Format {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Error used when a format version is not supported.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub struct UnsupportedFormatError {
    version: Format,
}

impl UnsupportedFormatError {
    /// Gets the format version that was not supported.
    pub fn version(&self) -> Format {
        self.version
    }
}

impl Display for UnsupportedFormatError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "format version {} is not supported, minimum supported is {}",
            self.version,
            SupportedFormat::MINIMUM
        )
    }
}

/// Represents an IL4IL binary format version number that is supported by this version of the API.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct SupportedFormat(Format);

impl SupportedFormat {
    /// The current format version, used by newly created modules.
    pub const CURRENT: Self = Self(Format { major: 0, minor: 1 });

    /// The minimum version that is supported, modules that use format versions lower than this cannot be read.
    pub const MINIMUM: Self = Self::CURRENT;

    /// Gets the underlying format version.
    #[must_use]
    pub const fn format(self) -> Format {
        self.0
    }

    /// Attempts to create a format version number, returning `Err` if the version is not supported.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::versioning::{Format, SupportedFormat};
    /// assert!(SupportedFormat::new(Format::new(0, 0)).is_err());
    /// ```
    pub fn new(version: Format) -> Result<Self, UnsupportedFormatError> {
        if version >= Self::MINIMUM.0 && version <= Self::CURRENT.0 {
            Ok(Self(version))
        } else {
            Err(UnsupportedFormatError { version })
        }
    }
}

impl TryFrom<Format> for SupportedFormat {
    type Error = UnsupportedFormatError;

    fn try_from(version: Format) -> Result<Self, Self::Error> {
        Self::new(version)
    }
}

impl Display for SupportedFormat {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
