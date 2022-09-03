//! Contains types representing configuration options.

use crate::loader::environment::Context as LoaderContext;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    #[cfg(target_endian = "little")]
    pub const HOST: Self = Self::Little;

    #[cfg(target_endian = "big")]
    pub const HOST: Self = Self::Big;
}

impl Default for Endianness {
    fn default() -> Self {
        Self::Little
    }
}

/// Provides configuration options for the IL4IL virtual machine.
#[derive(Debug)]
#[non_exhaustive]
pub struct Configuration {
    pub endianness: Endianness,
    pub loader_context: LoaderContext,
}

impl Configuration {
    pub const HOST: Self = Self {
        endianness: Endianness::HOST,
        loader_context: LoaderContext::HOST,
    };
}

impl Default for Configuration {
    fn default() -> Self {
        Self::HOST
    }
}
