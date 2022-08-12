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
    /// The file offset used when reporting an error.
    previous_file_offset: usize,
    next_file_offset: usize,
    locations: Vec<Location>,
}

impl<R: Read> Source<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            previous_file_offset: 0,
            next_file_offset: 0,
            locations: Vec::new(),
        }
    }

    /// Creates an [`Error`] with the current location and file offset information.
    #[must_use]
    pub fn create_error<E: Into<ErrorKind>>(&self, kind: E) -> Error {
        Error(Box::new(ErrorInner {
            kind: kind.into(),
            file_offset: self.previous_file_offset,
            locations: self.locations.clone().into_boxed_slice(),
        }))
    }

    /// The file offset of next byte that will be read next.
    pub fn next_file_offset(&self) -> usize {
        self.next_file_offset
    }

    /// Ensures that the current value of [`next_file_offset`](Source::next_file_offset) is used in error reporting.
    pub fn save_file_offset(&mut self) {
        let advanced_amount = self.next_file_offset - self.previous_file_offset;
        self.previous_file_offset = self.next_file_offset;
        for location in self.locations.iter_mut() {
            location.offset += advanced_amount;
        }
    }

    /// Pushes a named location onto the stack, using the file offset of the byte that will be read next.
    pub fn push_location(&mut self, name: &'static str) {
        self.locations.push(Location {
            name,
            starting_file_offset: self.next_file_offset,
            offset: 0,
        })
    }

    pub fn pop_location(&mut self) {
        self.locations.pop();
    }

    fn advance_location(&mut self, amount: usize) {
        self.next_file_offset += amount;
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
        self.source.read_exact(buf)
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
struct ErrorInner {
    kind: ErrorKind,
    file_offset: usize,
    locations: Box<[Location]>,
}

#[derive(thiserror::Error)]
pub struct Error(Box<ErrorInner>);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    /// A file offset to the location where an error occured.
    pub fn file_offset(&self) -> usize {
        self.0.file_offset
    }

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

pub type Result<T> = std::result::Result<T, Error>;

pub trait ReadFrom: Sized {
    /// Reads data from a source.
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self>;
}

impl ReadFrom for crate::binary::Module<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        todo!()
    }
}
