//! Low-level module for writing the contents of an IL4IL module.

use crate::function;
use crate::integer::VarU28;
use crate::module::section::{self, Section};
use crate::type_system::{self, TypeTag};
use std::io::{Error, ErrorKind, Write};
use std::ops::{Deref, DerefMut};

/// The result of writing to a stream of bytes.
pub type Result = std::io::Result<()>;

/// Provides a stream of bytes that can be written to.
#[derive(Debug)]
pub struct Destination<W: Write> {
    destination: W,
    buffers: Vec<Vec<u8>>,
}

impl<W: Write> Destination<W> {
    pub fn new(destination: W) -> Self {
        Self {
            destination,
            buffers: Default::default(),
        }
    }

    /// Gets a [`Destination`] to a byte buffer that bytes can be written to.
    ///
    /// This allows the writing of data in cases where the length of the bytes is not known beforehand.
    fn rent_buffer(&mut self) -> Destination<Vec<u8>> {
        let mut buffer_store = std::mem::take(&mut self.buffers);
        let buffer = buffer_store.pop().unwrap_or_default();
        Destination {
            destination: buffer,
            buffers: buffer_store,
        }
    }

    fn return_buffer(&mut self, mut buffers: Destination<Vec<u8>>) {
        self.buffers.append(&mut buffers.buffers);
        self.buffers.push(buffers.destination);
    }
}

impl<W: Write> Deref for Destination<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.destination
    }
}

impl<W: Write> DerefMut for Destination<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.destination
    }
}

impl<W: Write> From<W> for Destination<W> {
    fn from(destination: W) -> Self {
        Self::new(destination)
    }
}

impl<W: Write> Write for Destination<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.destination.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.destination.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.destination.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.destination.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.destination.write_fmt(fmt)
    }
}

/// A trait for writing a data into a destination.
pub trait WriteTo {
    /// Writes the data to a destination.
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result;
}

impl<T: WriteTo + Copy> WriteTo for &T {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        <T>::write_to(*self, out)
    }
}

impl WriteTo for u8 {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        out.write_all(&[self])
    }
}

impl WriteTo for VarU28 {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        VarU28::write_to(self, out)
    }
}

fn write_length(length: usize, out: &mut impl Write) -> Result {
    match VarU28::try_from(length) {
        Ok(value) => value.write_to(out),
        Err(e) => Err(Error::new(ErrorKind::InvalidInput, e)),
    }
}

impl WriteTo for &[u8] {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(self.len(), out)?;
        out.write_all(self)
    }
}

impl WriteTo for &crate::identifier::Id {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
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
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        let iter = self.0.into_iter();
        write_length(iter.len(), out)?;
        for item in iter {
            item.write_to(out)?;
        }
        Ok(())
    }
}

impl WriteTo for &section::Metadata<'_> {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        u8::from(self.kind()).write_to(out)?;
        match self {
            section::Metadata::Name(name) => name.write_to(out),
        }
    }
}

impl WriteTo for TypeTag {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        u8::from(self).write_to(out)
    }
}

impl WriteTo for type_system::SizedInteger {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        if let Some(size) = self.size() {
            let signed = self.sign().unwrap().is_signed();
            match size.bit_width().get() {
                8 => {
                    if signed {
                        TypeTag::S8.write_to(out)
                    } else {
                        TypeTag::U8.write_to(out)
                    }
                }
                16 => {
                    if signed {
                        TypeTag::S16.write_to(out)
                    } else {
                        TypeTag::U16.write_to(out)
                    }
                }
                32 => {
                    if signed {
                        TypeTag::S32.write_to(out)
                    } else {
                        TypeTag::U32.write_to(out)
                    }
                }
                64 => {
                    if signed {
                        TypeTag::S16.write_to(out)
                    } else {
                        TypeTag::U64.write_to(out)
                    }
                }
                128 => {
                    if signed {
                        TypeTag::S16.write_to(out)
                    } else {
                        TypeTag::U128.write_to(out)
                    }
                }
                256 => {
                    if signed {
                        TypeTag::S16.write_to(out)
                    } else {
                        TypeTag::U256.write_to(out)
                    }
                }
                size => {
                    if signed {
                        TypeTag::SInt.write_to(out)?;
                    } else {
                        TypeTag::UInt.write_to(out)?;
                    }

                    VarU28::write_to(VarU28::from_u16(size), out)
                }
            }
        } else {
            TypeTag::Bool.write_to(out)
        }
    }
}

impl WriteTo for type_system::Integer {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        match self {
            Self::Sized(sized) => sized.write_to(out),
            Self::Address(sign) => {
                if sign.is_signed() {
                    TypeTag::SAddr.write_to(out)
                } else {
                    TypeTag::UAddr.write_to(out)
                }
            }
        }
    }
}

impl WriteTo for type_system::Float {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        let tag = match self.bit_width().get() {
            16 => TypeTag::F16,
            32 => TypeTag::F32,
            64 => TypeTag::F64,
            128 => TypeTag::F128,
            256 => TypeTag::F256,
            bad => unimplemented!("unsupported float bit width: {bad}"),
        };

        tag.write_to(out)
    }
}

impl WriteTo for type_system::Type {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        match self {
            Self::Integer(i) => i.write_to(out),
            Self::Float(f) => f.write_to(out),
        }
    }
}

impl WriteTo for &type_system::Reference {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        match self {
            type_system::Reference::Index(index) => write_length(usize::from(*index), out),
            type_system::Reference::Inline(ty) => ty.write_to(out),
        }
    }
}

impl WriteTo for &function::Signature {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(self.result_type_count(), out)?;
        write_length(self.parameter_type_count(), out)?;
        for ty in self.all_types() {
            ty.write_to(out)?;
        }

        Ok(())
    }
}

impl WriteTo for &Section<'_> {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        u8::from(self.kind()).write_to(out)?;
        let mut section_buffer = out.rent_buffer();

        {
            let section_writer = &mut section_buffer;
            match self {
                Section::Metadata(metadata) => LengthPrefixed::from(metadata).write_to(section_writer)?,
                Section::Type(types) => LengthPrefixed::from(types.iter()).write_to(section_writer)?,
                Section::FunctionSignature(signatures) => LengthPrefixed::from(signatures).write_to(section_writer)?,
            }
        }

        write_length(section_buffer.len(), out)?;
        out.write_all(section_buffer.as_slice())?;
        out.return_buffer(section_buffer);
        Ok(())
    }
}

impl WriteTo for crate::versioning::Format {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        out.write_all(&[self.major, self.minor])
    }
}

impl WriteTo for &crate::module::Module<'_> {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        out.write_all(crate::binary::MAGIC.as_slice())?;
        self.format_version().version().write_to(out)?;
        LengthPrefixed::from(self.sections()).write_to(out)?;
        out.flush()
    }
}
