//! Low-level module for writing the contents of an IL4IL module.

use std::io::{Error, ErrorKind, Write};

pub type Result = std::io::Result<()>;

/// A trait for writing a data into a destination.
pub trait WriteTo {
    /// Writes the data to a destination.
    fn write_to<W: Write>(self, out: &mut W) -> Result;
}

impl<T: WriteTo + Copy> WriteTo for &T {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        <T>::write_to(*self, out)
    }
}

impl WriteTo for u8 {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        out.write_all(&[self])
    }
}

impl WriteTo for crate::integer::VarU28 {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        crate::integer::VarU28::write_to(self, out)
    }
}

fn write_length(length: usize, out: &mut impl Write) -> Result {
    match crate::integer::VarU28::try_from(length) {
        Ok(value) => value.write_to(out),
        Err(e) => Err(Error::new(ErrorKind::InvalidInput, e)),
    }
}

impl WriteTo for &[u8] {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        write_length(self.len(), out)?;
        out.write_all(self)
    }
}

impl WriteTo for &crate::identifier::Id {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        self.as_bytes().write_to(out)
    }
}

#[derive(Clone, Debug)]
pub struct LengthPrefixed<T>(T);

impl<T> From<T> for LengthPrefixed<T>
where
    T: IntoIterator,
    T::IntoIter: ExactSizeIterator,
    T::Item: WriteTo,
{
    fn from(items: T) -> Self {
        Self(items)
    }
}

impl<T> WriteTo for LengthPrefixed<T>
where
    T: IntoIterator,
    T::IntoIter: ExactSizeIterator,
    T::Item: WriteTo,
{
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        let iter = self.0.into_iter();
        write_length(iter.len(), out)?;
        for item in iter {
            item.write_to(out)?;
        }
        Ok(())
    }
}

impl WriteTo for &crate::binary::section::Metadata<'_> {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        u8::from(self.kind()).write_to(out)?;
        match self {
            crate::binary::section::Metadata::Name(name) => name.write_to(out),
        }
    }
}

impl WriteTo for &crate::binary::section::Section<'_> {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        u8::from(self.kind()).write_to(out)?;
        match self {
            crate::binary::section::Section::Metadata(metadata) => LengthPrefixed::from(metadata).write_to(out),
        }
    }
}

impl WriteTo for crate::versioning::Format {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        out.write_all(&[self.major, self.minor])
    }
}

impl WriteTo for &crate::binary::Module<'_> {
    fn write_to<W: Write>(self, out: &mut W) -> Result {
        out.write_all(crate::binary::MAGIC.as_slice())?;
        self.format_version().version().write_to(out)?;
        LengthPrefixed::from(self.sections()).write_to(out)
    }
}
