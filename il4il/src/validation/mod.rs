//! Module to perform validation of IL4IL code.
//!
//! Validation ensures that the contents of an IL4IL module are semantically correct. Additionally, validation does not require the
//! resolution of any imports.

#![deny(unsafe_code)]

mod contents;
mod error;

pub use contents::ModuleContents;
pub use error::*;

/// Represents a validated SAILAR module.
#[derive(Clone, Default)]
pub struct ValidModule<'data> {
    contents: ModuleContents<'data>,
}

impl<'data> ValidModule<'data> {
    /// Creates a valid module with the specified `contents`, without actually performing any validation.
    ///
    /// Using an invalid module may result in panics later.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the module is valid, though an invalid module will at worst only result in panics.
    #[allow(unsafe_code)]
    #[must_use]
    pub unsafe fn from_contents_unchecked(contents: ModuleContents<'data>) -> Self {
        Self { contents }
    }

    pub fn contents(&self) -> &ModuleContents<'data> {
        &self.contents
    }

    pub fn into_contents(self) -> ModuleContents<'data> {
        self.contents
    }

    /// Validates the given module contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the module contents are invalid.
    pub fn from_module_contents(contents: ModuleContents<'data>) -> Result<Self, Error> {
        // TODO: Add validation
        Ok(Self { contents })
    }
}

impl<'data> TryFrom<ModuleContents<'data>> for ValidModule<'data> {
    type Error = Error;

    fn try_from(value: ModuleContents<'data>) -> Result<Self, Error> {
        Self::from_module_contents(value)
    }
}

impl<'data> TryFrom<crate::binary::Module<'data>> for ValidModule<'data> {
    type Error = Error;

    fn try_from(value: crate::binary::Module<'data>) -> Result<Self, Error> {
        Self::from_module_contents(ModuleContents::from_module(value))
    }
}
