//! Contains types that model the contents of a valid IL4IL module.

use crate::binary::section::{self, Section};

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

    pub fn from_sections_fallible<S, E>(sections: S) -> Result<Self, E>
    where
        S: IntoIterator<Item = Result<Section<'data>, E>>
    {
        let mut contents = ModuleContents::default();

        for sect in sections.into_iter() {
            match sect? {
                Section::Metadata(mut metadata) => contents.metadata.append(&mut metadata),
            }
        }

        Ok(contents)
    }

    pub fn from_sections<S: IntoIterator<Item = Section<'data>>>(sections: S) -> Self {
        Self::from_sections_fallible(sections.into_iter().map(Result::<_, std::convert::Infallible>::Ok)).unwrap()
    }

    pub fn from_module(module: crate::binary::Module<'data>) -> Self {
        Self::from_sections(module.into_sections())
    }
}
