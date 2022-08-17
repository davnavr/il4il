//! Contains types that model the structure of an IL4IL module.

use crate::binary::parser;
use crate::binary::writer;
use crate::versioning::SupportedFormat;

pub mod section;

use section::Section;

/// An in-memory representation of an IL4IL module.
#[derive(Clone, Debug)]
pub struct Module<'data> {
    format_version: SupportedFormat,
    sections: Vec<Section<'data>>,
}

impl<'data> Module<'data> {
    #[must_use]
    pub(crate) fn with_format_version_and_sections(format_version: SupportedFormat, sections: Vec<Section<'data>>) -> Self {
        Self { format_version, sections }
    }

    /// Creates an empty module with the current format version.
    #[must_use]
    pub fn new() -> Self {
        Self::with_format_version_and_sections(SupportedFormat::CURRENT, Vec::new())
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

    /// Writes the binary contents of the module to the specified destination.
    pub fn write_to<W: std::io::Write>(&self, destination: W) -> writer::Result {
        writer::WriteTo::write_to(self, &mut writer::Destination::new(destination))
    }

    /// Writes the binary contents of a module to the specified [`Path`].
    ///
    /// [`Path`]: std::path::Path
    pub fn write_to_path<P: AsRef<std::path::Path>>(&self, path: P) -> writer::Result {
        self.write_to(std::io::BufWriter::new(
            std::fs::OpenOptions::new().write(true).truncate(true).open(path)?,
        ))
    }

    /// Reads the binary contents of a module from the specified source.
    ///
    /// # Examples
    ///
    /// ```
    /// # use il4il::module::*;
    /// assert!(matches!(Module::read_from([ 1u8, 2, 3, 4 ].as_slice()), Err(e) if e.file_offset() == 0));
    /// ```
    pub fn read_from<R: std::io::Read>(source: R) -> parser::Result<Self> {
        let mut reader = parser::Source::new(source);
        <Self as parser::ReadFrom>::read_from(&mut reader)
    }
}

impl Default for Module<'_> {
    fn default() -> Self {
        Self::new()
    }
}