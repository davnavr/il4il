//! Module for parsing the contents of an IL4IL module.

use std::fmt::{Debug, Display, Formatter};
use std::io::Read;

/// Marks a named location within a file.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Location {
    pub name: &'static str,
    /// A file offset indicating the start of this named location.
    pub starting_file_offset: usize,
    /// An offset from the start of this named location.
    pub offset: usize,
}

/// Provides a stream of bytes, keeping track of location and offset information.
#[derive(Debug)]
pub struct Source<R: Read> {
    source: R,
    saved_file_offset: usize,
    file_offset: usize,
    locations: Vec<Location>,
}

impl<R: Read> Source<R> {
    /// Creates a [`Source<R>`](Source) from an [`io::Read`](std::io::Read) instance.
    pub fn new(source: R) -> Self {
        Self {
            source,
            saved_file_offset: 0,
            file_offset: 0,
            locations: Vec::new(),
        }
    }

    /// Creates an [`Error`] with the current location and file offset information.
    #[must_use]
    pub fn create_error<E: Into<ErrorKind>>(&self, kind: E) -> Error {
        Error(Box::new(ErrorInner {
            kind: kind.into(),
            file_offset: self.saved_file_offset,
            locations: self.locations.clone().into_boxed_slice(),
        }))
    }

    /// The file offset of next byte that will be read next.
    pub fn file_offset(&self) -> usize {
        self.file_offset
    }

    /// Saves the current file offset for use in error reporting.
    pub fn save_file_offset(&mut self) {
        let advanced_amount = self.file_offset - self.saved_file_offset;
        if advanced_amount > 0 {
            self.saved_file_offset = self.file_offset;
            for location in self.locations.iter_mut() {
                location.offset += advanced_amount;
            }
        }
    }

    /// Pushes a named location onto the location stack, using the file offset of the byte that will be read next.
    pub fn push_location(&mut self, name: &'static str) {
        self.locations.push(Location {
            name,
            starting_file_offset: self.file_offset,
            offset: 0,
        })
    }

    pub fn pop_location(&mut self) {
        self.locations.pop();
    }

    fn advance_location(&mut self, amount: usize) {
        self.file_offset += amount;
    }
}

impl<R: Read> From<R> for Source<R> {
    fn from(source: R) -> Self {
        Self::new(source)
    }
}

impl<R: Read> Read for Source<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amount = self.source.read(buf)?;
        self.advance_location(amount);
        Ok(amount)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.source.read_exact(buf)?;
        self.advance_location(buf.len());
        Ok(())
    }
}

/// Error type used when a file does not start with the IL4IL [module magic value](crate::binary::MAGIC).
#[derive(Clone, Debug, thiserror::Error)]
#[error("not a valid IL4IL module file")]
#[non_exhaustive]
pub struct InvalidMagicError;

/// Error type used when an unsigned integer length cannot be used.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0} is not a valid length value")]
pub struct LengthIntegerError(crate::integer::VarU28);

/// Error type indicating why parsing failed. Used with the [`Error`] type.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error(transparent)]
    InvalidMagic(#[from] InvalidMagicError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    UnsupportedFormat(#[from] crate::versioning::UnsupportedFormatError),
    #[error(transparent)]
    UnsupportedIntegerLength(#[from] crate::integer::LengthError),
    #[error(transparent)]
    BadLengthInteger(#[from] LengthIntegerError),
    #[error(transparent)]
    InvalidSectionKind(#[from] crate::binary::section::SectionKindError),
}

#[derive(Debug)]
struct ErrorInner {
    kind: ErrorKind,
    file_offset: usize,
    locations: Box<[Location]>,
}

/// Error type used when parsing fails.
#[derive(thiserror::Error)]
pub struct Error(Box<ErrorInner>);

impl Error {
    /// The kind of error encountered during parsing.
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    /// A file offset to the location where an error occured.
    pub fn file_offset(&self) -> usize {
        self.0.file_offset
    }

    /// Locations of interest encountered during parsing.
    pub fn locations(&self) -> &[Location] {
        &self.0.locations
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "error occurred at file offset {:#X}: {}", self.0.file_offset, self.0.kind)?;
        for location in self.locations() {
            writeln!(
                f,
                "  at offset {:#X} from {:?} which starts at file offset {:#X}",
                location.offset, location.name, location.starting_file_offset
            )?;
        }
        Ok(())
    }
}

/// A [`Result`] type used by parsers.
pub type Result<T> = std::result::Result<T, Error>;

/// A trait for parsing data from a stream of bytes.
pub trait ReadFrom: Sized {
    /// Reads data from a source.
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self>;

    /// Reads a contiguous sequence of data from a source.
    fn read_many<R: Read>(source: &mut Source<R>, count: usize) -> Result<Box<[Self]>> {
        let mut data = Vec::with_capacity(count);
        for _ in 0..count {
            source.save_file_offset();
            data.push(Self::read_from(source)?);
        }

        Ok(data.into_boxed_slice())
    }
}

impl ReadFrom for u8 {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let mut value = 0u8;
        source.save_file_offset();
        source
            .read_exact(std::slice::from_mut(&mut value))
            .map_err(|e| source.create_error(e))?;
        Ok(value)
    }

    fn read_many<R: Read>(source: &mut Source<R>, count: usize) -> Result<Box<[Self]>> {
        if count == 0 {
            return Ok(Default::default());
        }

        let mut data = vec![0u8; count].into_boxed_slice();
        source.save_file_offset();
        source.read_exact(&mut data).map_err(|e| source.create_error(e))?;
        Ok(data)
    }
}

impl ReadFrom for crate::versioning::Format {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        source.save_file_offset();
        let mut bytes = [0u8; 2];
        source.read_exact(&mut bytes).map_err(|e| source.create_error(e))?;
        Ok(Self::new(bytes[0], bytes[1]))
    }
}

impl ReadFrom for crate::versioning::SupportedFormat {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        Self::try_from(crate::versioning::Format::read_from(source)?).map_err(|e| source.create_error(e))
    }
}

impl ReadFrom for crate::integer::VarU28 {
    fn read_from<R: Read>(mut source: &mut Source<R>) -> Result<Self> {
        match Self::read_from(&mut source) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(source.create_error(error)),
            Err(error) => Err(source.create_error(error)),
        }
    }
}

fn parse_length(src: &mut Source<impl Read>) -> Result<usize> {
    src.save_file_offset();
    let value = <crate::integer::VarU28 as ReadFrom>::read_from(src)?;
    usize::try_from(value).map_err(|_| src.create_error(LengthIntegerError(value)))
}

fn parse_many_length_encoded<T: ReadFrom, R: Read>(src: &mut Source<R>) -> Result<Box<[T]>> {
    let count = parse_length(src)?;
    T::read_many(src, count)
}

impl ReadFrom for crate::binary::section::Section<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        match crate::binary::section::SectionKind::try_from(u8::read_from(source)?).map_err(|e| source.create_error(e))? {
            _ => todo!(),
        }
    }
}

impl<'data> ReadFrom for crate::binary::Module<'data> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        {
            source.save_file_offset();
            let mut magic_buffer = [0u8; crate::binary::MAGIC.len()];
            let count = source.read(&mut magic_buffer).map_err(|e| source.create_error(e))?;
            let actual_magic = &magic_buffer[0..count];
            if actual_magic != crate::binary::MAGIC.as_slice() {
                return Err(source.create_error(InvalidMagicError));
            }
        }

        source.push_location("format version");
        let format_version = crate::versioning::SupportedFormat::read_from(source)?;
        source.pop_location();

        source.push_location("sections");
        let sections = parse_many_length_encoded::<crate::binary::section::Section<'data>, _>(source)?;
        source.pop_location();

        Ok(Self::with_format_version_and_sections(format_version, sections.into_vec()))
    }
}
