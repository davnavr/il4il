use crate::binary::section::Section;
use crate::versioning::SupportedFormat;

// An in-memory representation of an IL4IL module.
#[derive(Clone, Debug)]
pub struct Module<'data> {
    format_version: SupportedFormat,
    sections: Vec<Section<'data>>,
}

impl<'data> Module<'data> {
    /// Creates an empty module with the current format version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            format_version: SupportedFormat::CURRENT,
            sections: Vec::new(),
        }
    }

    /// Returns the format version of the module.
    #[must_use]
    pub fn format_version(&self) -> SupportedFormat {
        self.format_version
    }

    /// Returns a reference module's sections.
    #[must_use]
    pub fn sections(&self) -> &Vec<Section<'data>> {
        &self.sections
    }

    /// Returns a mutable reference to the module's sections.
    #[must_use]
    pub fn sections_mut(&mut self) -> &mut Vec<Section<'data>> {
        &mut self.sections
    }

    /// Returns the module's sections.
    #[must_use]
    pub fn into_sections(self) -> Vec<Section<'data>> {
        self.sections
    }

    #[must_use]
    pub fn into_owned<'owned>(self) -> Module<'owned> {
        Module {
            format_version: self.format_version,
            sections: self.sections.into_iter().map(Section::into_owned).collect(),
        }
    }
}

impl Default for Module<'_> {
    fn default() -> Self {
        Self::new()
    }
}
