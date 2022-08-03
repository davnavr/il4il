use crate::binary::section::Section;
use crate::versioning::SupportedFormat;

// An in-memory representation of an IL4IL module.
#[derive(Clone, Debug)]
pub struct Module {
    format_version: SupportedFormat,
    sections: Vec<Section>,
}

impl Module {
    /// Creates an empty module with the current format version.
    pub fn new() -> Self {
        Self { format_version: SupportedFormat::CURRENT, sections: Vec::new() }
    }

    /// Returns the format version of the module.
    pub fn format_version(&self) -> SupportedFormat {
        self.format_version
    }

    /// Returns a reference module's sections.
    pub fn sections(&self) -> &Vec<Section> {
        &self.sections
    }

    /// Returns a mutable reference to the module's sections.
    pub fn sections_mut(&mut self) -> &mut Vec<Section> {
        &mut self.sections
    }

    /// Appends a section to the end of this module.
    pub fn append_section<S: Into<Section>>(&mut self, section: S) {
        self.sections.push(section.into())
    }

    /// Returns the module's sections.
    pub fn into_sections(self) -> Vec<Section> {
        self.sections
    }
}
