//! Module to manipulate IL4IL module sections.

/// Represents an IL4IL module section.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Section {
    /// The metadata section contains information about the module.
    Metadata(()),
}
