//! Contains types that model the contents of a valid IL4IL module.

use crate::binary::section;

/// Represents the contents of a SAILAR module.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ModuleContents<'data> {
    pub metadata: Vec<section::Metadata<'data>>,
}

impl<'data> ModuleContents<'data> {
    /// Creates an empty SAILAR module.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
