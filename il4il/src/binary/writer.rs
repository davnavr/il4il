//! Low-level module for writing the contents of an IL4IL module.

use crate::function;
use crate::identifier::{self, Id};
use crate::instruction::value::{self, Value};
use crate::instruction::{self, Instruction};
use crate::integer::{VarI28, VarU28};
use crate::module::section::{self, Section};
use crate::symbol;
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

impl<const N: usize> WriteTo for &[u8; N] {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        out.write_all(self.as_slice())
    }
}

impl WriteTo for VarU28 {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        <VarU28>::write_to(self, out)
    }
}

impl WriteTo for VarI28 {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        <VarI28>::write_to(self, out)
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

impl WriteTo for &Id {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        self.as_bytes().write_to(out)
    }
}

impl WriteTo for &identifier::Identifier {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        <&Id>::write_to(self.as_id(), out)
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

impl WriteTo for &symbol::Assignment<'_> {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        out.write_all([u8::from(self.target_kind()), u8::from(self.symbol_kind())].as_slice())?;

        write_length(self.symbols.len(), out)?;
        for (symbol, index) in self.symbols.iter() {
            <&'_ Id>::write_to(symbol, out)?;
            write_length(*index, out)?;
        }

        Ok(())
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

impl WriteTo for &type_system::Type {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        match self {
            type_system::Type::Integer(i) => i.write_to(out),
            type_system::Type::Float(f) => f.write_to(out),
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
        self.all_types().iter().try_for_each(|ty| ty.write_to(out))
    }
}

impl WriteTo for &function::Instantiation {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(usize::from(self.template), out)?;
        <VarU28 as WriteTo>::write_to(VarU28::from_u8(0), out)
    }
}

impl WriteTo for &function::Import {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(usize::from(self.module), out)?;
        write_length(usize::from(self.signature), out)
    }
}

impl WriteTo for &function::Definition {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(usize::from(self.signature), out)?;
        write_length(usize::from(self.body), out)
    }
}

impl WriteTo for &Value {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        use value::{Constant, ConstantFloat, ConstantInteger};

        match self {
            Value::Constant(constant) => {
                WriteTo::write_to(VarI28::from(constant.tag()), out)?;
                match constant {
                    Constant::Integer(
                        ConstantInteger::All
                        | ConstantInteger::One
                        | ConstantInteger::SignedMaximum
                        | ConstantInteger::SignedMinimum
                        | ConstantInteger::Zero,
                    ) => Ok(()),
                    Constant::Integer(ConstantInteger::Byte(value)) => value.write_to(out),
                    Constant::Integer(ConstantInteger::I16(bytes)) | Constant::Float(ConstantFloat::Half(bytes)) => bytes.write_to(out),
                    Constant::Integer(ConstantInteger::I32(bytes)) | Constant::Float(ConstantFloat::Single(bytes)) => bytes.write_to(out),
                    Constant::Integer(ConstantInteger::I64(bytes)) | Constant::Float(ConstantFloat::Double(bytes)) => bytes.write_to(out),
                    Constant::Integer(ConstantInteger::I128(bytes)) | Constant::Float(ConstantFloat::Quadruple(bytes)) => {
                        bytes.write_to(out)
                    }
                }
            }
        }
    }
}

impl WriteTo for &Instruction {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        WriteTo::write_to(VarU28::from(self.opcode()), out)?;
        match self {
            Instruction::Unreachable => Ok(()),
            Instruction::Return(values) => LengthPrefixed::from(values.iter()).write_to(out),
        }
    }
}

impl WriteTo for &instruction::Block {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        write_length(self.input_count(), out)?;
        write_length(self.temporary_count(), out)?;
        self.types.iter().try_for_each(|ty| ty.write_to(out))?;
        // TODO: Include byte length of instructions?
        LengthPrefixed::from(self.instructions.iter()).write_to(out)
    }
}

impl WriteTo for &function::Body {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        LengthPrefixed::from(self.result_types.iter()).write_to(out)?;
        write_length(self.other_blocks.len(), out)?;
        self.entry_block.write_to(out)?;
        self.other_blocks.iter().try_for_each(|block| block.write_to(out))
    }
}

impl WriteTo for &crate::module::ModuleName<'_> {
    fn write_to<W: Write>(self, out: &mut Destination<W>) -> Result {
        self.name.write_to(out)?;
        VarU28::MIN.write_to(out)
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
                Section::Symbol(symbols) => LengthPrefixed::from(symbols).write_to(section_writer)?,
                Section::Type(types) => LengthPrefixed::from(types).write_to(section_writer)?,
                Section::FunctionSignature(signatures) => LengthPrefixed::from(signatures).write_to(section_writer)?,
                Section::FunctionInstantiation(instantiations) => LengthPrefixed::from(instantiations).write_to(section_writer)?,
                Section::FunctionImport(imports) => LengthPrefixed::from(imports).write_to(section_writer)?,
                Section::FunctionDefinition(definitions) => LengthPrefixed::from(definitions).write_to(section_writer)?,
                Section::Code(code) => LengthPrefixed::from(code).write_to(section_writer)?,
                Section::EntryPoint(index) => write_length(usize::from(*index), section_writer)?,
                Section::ModuleImport(modules) => LengthPrefixed::from(modules).write_to(section_writer)?,
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
