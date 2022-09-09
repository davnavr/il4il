//! Module for reading from a stream of bytes.

use crate::binary::parser::{Error, Result};
use error_stack::{IntoReport, ResultExt};
use std::io::Read;

/// Provides a stream of bytes, keeping track of location and offset information.
#[derive(Debug)]
pub struct Source<R: Read> {
    source: R,
    file_offset: usize,
}

impl<R: Read> Source<R> {
    /// Creates a [`Source<R>`](Source) from an [`io::Read`](std::io::Read) instance.
    #[must_use]
    pub fn new(source: R) -> Self {
        Self { source, file_offset: 0 }
    }

    /// The file offset of the byte that will be read next.
    pub fn file_offset(&self) -> usize {
        self.file_offset
    }

    /// Attempts to read the exact number of bytes to fill the `buffer`.
    pub fn fill_buffer(&mut self, buffer: &mut [u8]) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        let offset = self.file_offset;
        let length = Read::read(self, buffer)
            .report()
            .change_context_lazy(|| Error::new(offset))
            .attach_printable_lazy(|| format!("expected {} bytes", buffer.len()))?;
        if length != buffer.len() {
            return Err(Error::new(offset))
                .report()
                .attach_printable_lazy(|| format!("expected {} bytes but got {}", buffer.len(), length));
        }
        Ok(())
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
        self.file_offset += amount;
        Ok(amount)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.source.read_exact(buf)?;
        self.file_offset += buf.len();
        Ok(())
    }
}
