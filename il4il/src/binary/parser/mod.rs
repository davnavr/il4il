//! Module for parsing the contents of an IL4IL module.

use crate::function;
use crate::identifier::Identifier;
use crate::index;
use crate::instruction;
use crate::integer;
use crate::module;
use crate::module::section;
use crate::symbol;
use crate::type_system;
use error_stack::{IntoReport, ResultExt};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Read;

mod error;
mod source;

pub use error::{Error, Report};
pub use source::Source;

/// Trait implemented by types representing bit flags or tags.
trait FlagsValue: Sized {
    type Value: std::fmt::UpperHex + Copy;

    fn name() -> &'static str;

    fn from_value(value: Self::Value) -> Option<Self>;
}

macro_rules! flags_values {
    ($($implementor:ty : $integer:ty, name = $name:literal;)*) => {
        $(impl FlagsValue for $implementor {
            type Value = $integer;

            fn name() -> &'static str {
                $name
            }

            fn from_value(value: Self::Value) -> Option<Self> {
                Self::new(value)
            }
        })*
    };
}

flags_values! {
    section::SectionKind : u8, name = "section kind";
    section::MetadataKind : u8, name = "metadata kind";
    symbol::Kind : u8, name = "symbol kind";
    symbol::TargetKind : u8, name = "symbol target";
}

/// A specialized [`Result`] type returned by parser methods.
///
/// [`Result`]: std::result::Result
pub type Result<T> = std::result::Result<T, Report>;

/// A trait for parsing data from a stream of bytes.
pub trait ReadFrom: Sized {
    /// Reads data from a source.
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self>;

    /// Reads a contiguous sequence of data from a source.
    fn read_many<R: Read>(source: &mut Source<R>, count: usize) -> Result<Box<[Self]>> {
        let mut data = Vec::with_capacity(count);
        for _ in 0..count {
            data.push(Self::read_from(source)?);
        }
        Ok(data.into_boxed_slice())
    }
}

impl ReadFrom for u8 {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let mut value = 0u8;
        source.fill_buffer(std::slice::from_mut(&mut value))?;
        Ok(value)
    }

    fn read_many<R: Read>(source: &mut Source<R>, count: usize) -> Result<Box<[Self]>> {
        if count == 0 {
            return Ok(Default::default());
        }

        let mut data = vec![0u8; count];
        source.fill_buffer(&mut data)?;
        Ok(data.into_boxed_slice())
    }
}

impl ReadFrom for crate::versioning::Format {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let mut bytes = [0u8; 2];
        source.fill_buffer(&mut bytes).attach_printable("malformed format version")?;
        Ok(Self::new(bytes[0], bytes[1]))
    }
}

impl ReadFrom for crate::versioning::SupportedFormat {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        Self::try_from(crate::versioning::Format::read_from(source)?)
            .report()
            .change_context_lazy(|| Error::new(offset))
    }
}

impl ReadFrom for integer::VarU28 {
    fn read_from<R: Read>(mut source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        match Self::read_from(&mut source) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(error).report().change_context_lazy(|| Error::new(offset)),
            Err(error) => Err(error)
                .report()
                .change_context_lazy(|| Error::new(offset))
                .attach_printable("malformed variable-length unsigned integer"),
        }
    }
}

impl ReadFrom for integer::VarI28 {
    fn read_from<R: Read>(mut source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        match Self::read_from(&mut source) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(error).report().change_context_lazy(|| Error::new(offset)),
            Err(error) => Err(error)
                .report()
                .change_context_lazy(|| Error::new(offset))
                .attach_printable("malformed variable-length signed integer"),
        }
    }
}

fn parse_length<L: From<usize>>(src: &mut Source<impl Read>) -> Result<L> {
    let offset = src.file_offset();
    let value = <integer::VarU28 as ReadFrom>::read_from(src)?;
    usize::try_from(value)
        .map(L::from)
        .report()
        .change_context_lazy(|| Error::new(offset))
}

fn parse_many_length_encoded<T: ReadFrom, R: Read>(src: &mut Source<R>) -> Result<Box<[T]>> {
    let count = parse_length(src).attach_printable("length")?;
    T::read_many(src, count)
}

fn parse_flags_value<T, R>(src: &mut Source<R>) -> Result<T>
where
    T: FlagsValue,
    T::Value: ReadFrom,
    R: Read,
{
    let offset = src.file_offset();
    let flags = <T::Value>::read_from(src).attach_printable(T::name())?;
    T::from_value(flags)
        .ok_or_else(|| Error::new(offset))
        .report()
        .attach_printable_lazy(|| format!("{flags:#02X} is not a valid {} value", T::name()))
}

impl ReadFrom for Identifier {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        let bytes = parse_many_length_encoded(source).attach_printable("identifier contents")?;
        Self::from_utf8(bytes.into_vec())
            .report()
            .change_context_lazy(|| Error::new(offset))
    }
}

impl ReadFrom for module::ModuleName<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let name = Identifier::read_from(source).attach_printable("module name")?;

        {
            let offset = source.file_offset();
            let reserved = parse_length::<usize>(source)?;
            if reserved != 0 {
                return Err(Error::new(offset))
                    .report()
                    .attach_printable("reserved integer after module name must be zero");
            }
        }

        Ok(Self::from_name(name))
    }
}

impl ReadFrom for section::Metadata<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let metadata = match parse_flags_value(source)? {
            section::MetadataKind::Name => section::Metadata::Name(module::ModuleName::read_from(source)?),
        };
        Ok(metadata)
    }
}

impl ReadFrom for symbol::Assignment<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let target_kind: symbol::TargetKind = parse_flags_value(source)?;
        let symbol_kind: symbol::Kind = parse_flags_value(source)?;
        let mut assignment = Self::new(symbol_kind, target_kind);
        let count: usize = parse_length(source).attach_printable("symbol count")?;
        for _ in 0..count {
            let name = Identifier::read_from(source).attach_printable("symbol name")?;
            let index: usize = parse_length(source).attach_printable("symbol assignment index")?;
            assignment.symbols.push((Cow::Owned(name), index));
        }

        Ok(assignment)
    }
}

impl ReadFrom for type_system::IntegerSize {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        Self::from_u28(<integer::VarU28 as ReadFrom>::read_from(source).attach_printable("integer size")?)
            .report()
            .change_context_lazy(|| Error::new(offset))
    }
}

impl ReadFrom for type_system::Reference {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        let tag_value = <integer::VarI28 as ReadFrom>::read_from(source).attach_printable("type reference tag")?;
        match integer::VarU28::try_from(tag_value) {
            Ok(index) => Ok(type_system::Reference::Index(index::Type::new(
                usize::try_from(index)
                    .report()
                    .change_context_lazy(|| Error::new(offset))
                    .attach_printable("indexed type")?,
            ))),
            Err(_) => {
                let offset = source.file_offset();
                let value = type_system::TypeTag::try_from(tag_value)
                    .report()
                    .change_context_lazy(|| Error::new(offset))?;
                let inline_type = match value {
                    type_system::TypeTag::Bool => type_system::Type::from(type_system::SizedInteger::BOOL),
                    type_system::TypeTag::U8 => type_system::Type::from(type_system::SizedInteger::U8),
                    type_system::TypeTag::S8 => type_system::Type::from(type_system::SizedInteger::S8),
                    type_system::TypeTag::U16 => type_system::Type::from(type_system::SizedInteger::U16),
                    type_system::TypeTag::S16 => type_system::Type::from(type_system::SizedInteger::S16),
                    type_system::TypeTag::U32 => type_system::Type::from(type_system::SizedInteger::U32),
                    type_system::TypeTag::S32 => type_system::Type::from(type_system::SizedInteger::S32),
                    type_system::TypeTag::U64 => type_system::Type::from(type_system::SizedInteger::U64),
                    type_system::TypeTag::S64 => type_system::Type::from(type_system::SizedInteger::S64),
                    type_system::TypeTag::U128 => type_system::Type::from(type_system::SizedInteger::U128),
                    type_system::TypeTag::S128 => type_system::Type::from(type_system::SizedInteger::S128),
                    type_system::TypeTag::U256 => type_system::Type::from(type_system::SizedInteger::U256),
                    type_system::TypeTag::S256 => type_system::Type::from(type_system::SizedInteger::S256),
                    type_system::TypeTag::UAddr => {
                        type_system::Type::Integer(type_system::Integer::Address(type_system::IntegerSign::UNSIGNED))
                    }
                    type_system::TypeTag::SAddr => {
                        type_system::Type::Integer(type_system::Integer::Address(type_system::IntegerSign::SIGNED))
                    }
                    type_system::TypeTag::UInt => type_system::Type::from(type_system::SizedInteger::new(
                        type_system::IntegerSign::UNSIGNED,
                        type_system::IntegerSize::read_from(source)?,
                    )),
                    type_system::TypeTag::SInt => type_system::Type::from(type_system::SizedInteger::new(
                        type_system::IntegerSign::SIGNED,
                        type_system::IntegerSize::read_from(source)?,
                    )),
                    type_system::TypeTag::F16 => type_system::Type::Float(type_system::Float::F16),
                    type_system::TypeTag::F32 => type_system::Type::Float(type_system::Float::F32),
                    type_system::TypeTag::F64 => type_system::Type::Float(type_system::Float::F64),
                    type_system::TypeTag::F128 => type_system::Type::Float(type_system::Float::F128),
                    type_system::TypeTag::F256 => type_system::Type::Float(type_system::Float::F256),
                };

                Ok(type_system::Reference::Inline(inline_type))
            }
        }
    }
}

impl ReadFrom for type_system::Type {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let offset = source.file_offset();
        match type_system::Reference::read_from(source)? {
            type_system::Reference::Inline(ty) => Ok(ty),
            type_reference => Err(Error::new(offset))
                .report()
                .attach_printable_lazy(|| format!("expected a type index but got an inline {type_reference}")),
        }
    }
}

impl ReadFrom for function::Signature {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let result_count: usize = parse_length(source)?;
        let parameter_count: usize = parse_length(source)?;
        type_system::Reference::read_many(source, result_count + parameter_count)
            .map(|types| function::Signature::from_types(types, result_count))
    }
}

impl ReadFrom for function::Instantiation {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let template = parse_length(source).attach_printable("function instantiation template index")?;
        let offset = source.file_offset();
        let reserved = parse_length::<usize>(source).attach_printable("reserved")?;
        if reserved != 0 {
            return Err(Error::new(offset))
                .report()
                .attach_printable("expected reserved integer in function instantiation to be zero");
        }

        Ok(Self::with_template(template))
    }
}

impl ReadFrom for function::Import<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        Ok(Self::new(
            parse_length(source).attach_printable("function import module index")?,
            Cow::Owned(Identifier::read_from(source).attach_printable("function import symbol")?),
            parse_length(source).attach_printable("function import signature index")?,
        ))
    }
}

impl ReadFrom for function::Definition {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let signature = parse_length(source).attach_printable("function definition signature index")?;
        let body = parse_length(source).attach_printable("function definition body index")?;

        let offset = source.file_offset();
        let reserved = parse_length::<usize>(source).attach_printable("reserved")?;
        if reserved != 0 {
            return Err(Error::new(offset))
                .report()
                .attach_printable("expected reserved integer in function definition to be zero");
        }

        Ok(Self::new(signature, body))
    }
}

impl ReadFrom for instruction::value::Value {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        use instruction::value::{ConstantFloat, ConstantInteger, ConstantTag};

        let offset = source.file_offset();
        let tag = <integer::VarI28 as ReadFrom>::read_from(source).attach_printable("value tag")?;
        if tag.get() >= 0 {
            todo!("parsing of register index values is not yet supported")
        } else {
            Ok(
                match ConstantTag::try_from(tag).report().change_context_lazy(|| Error::new(offset))? {
                    ConstantTag::IntegerZero => ConstantInteger::Zero.into(),
                    ConstantTag::IntegerOne => ConstantInteger::One.into(),
                    ConstantTag::IntegerAll => ConstantInteger::All.into(),
                    ConstantTag::IntegerSignedMaximum => ConstantInteger::SignedMaximum.into(),
                    ConstantTag::IntegerSignedMinimum => ConstantInteger::SignedMinimum.into(),
                    ConstantTag::IntegerInline8 => ConstantInteger::Byte(u8::read_from(source).attach_printable("constant byte")?).into(),
                    ConstantTag::IntegerInline16 => {
                        let mut bytes = [0u8; 2];
                        source.fill_buffer(&mut bytes).attach_printable("constant 16-bit integer")?;
                        ConstantInteger::I16(bytes).into()
                    }
                    ConstantTag::IntegerInline32 => {
                        let mut bytes = [0u8; 4];
                        source.fill_buffer(&mut bytes).attach_printable("constant 32-bit integer")?;
                        ConstantInteger::I32(bytes).into()
                    }
                    ConstantTag::IntegerInline64 => {
                        let mut bytes = [0u8; 8];
                        source.fill_buffer(&mut bytes).attach_printable("constant 64-bit integer")?;
                        ConstantInteger::I64(bytes).into()
                    }
                    ConstantTag::IntegerInline128 => {
                        let mut bytes = [0u8; 16];
                        source.fill_buffer(&mut bytes).attach_printable("constant 128-bit integer")?;
                        ConstantInteger::I128(bytes).into()
                    }
                    ConstantTag::Float16 => {
                        let mut bytes = [0u8; 2];
                        source.fill_buffer(&mut bytes).attach_printable("constant 16-bit float")?;
                        ConstantFloat::Half(bytes).into()
                    }
                    ConstantTag::Float32 => {
                        let mut bytes = [0u8; 4];
                        source.fill_buffer(&mut bytes).attach_printable("constant 32-bit float")?;
                        ConstantFloat::Single(bytes).into()
                    }
                    ConstantTag::Float64 => {
                        let mut bytes = [0u8; 8];
                        source.fill_buffer(&mut bytes).attach_printable("constant 64-bit float")?;
                        ConstantFloat::Double(bytes).into()
                    }
                    ConstantTag::Float128 => {
                        let mut bytes = [0u8; 16];
                        source.fill_buffer(&mut bytes).attach_printable("constant 128-bit float")?;
                        ConstantFloat::Quadruple(bytes).into()
                    }
                },
            )
        }
    }
}

impl ReadFrom for instruction::Instruction {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        use instruction::{Instruction, Opcode};

        let offset = source.file_offset();
        let opcode = <integer::VarU28 as ReadFrom>::read_from(source).attach_printable("opcode")?;
        Ok(
            match Opcode::try_from(opcode).report().change_context_lazy(|| Error::new(offset))? {
                Opcode::Unreachable => Instruction::Unreachable,
                Opcode::Return => Instruction::Return(parse_many_length_encoded(source).attach_printable("return values")?),
            },
        )
    }
}

impl ReadFrom for instruction::Block {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let input_count: usize = parse_length(source).attach_printable("block input count")?;
        let temporary_count: usize = parse_length(source).attach_printable("block temporary count")?;
        let types = type_system::Reference::read_many(source, input_count + temporary_count).attach_printable("block types")?;
        let instructions = parse_many_length_encoded(source)?;
        Ok(Self::from_types(types, input_count, instructions.into_vec()))
    }
}

impl ReadFrom for function::Body {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let result_types = parse_many_length_encoded(source).attach_printable("function body result types")?;
        let other_block_count: usize = parse_length(source).attach_printable("function body other block count")?;
        let entry_block = instruction::Block::read_from(source).attach_printable("entry block")?;
        let other_blocks = instruction::Block::read_many(source, other_block_count).attach_printable("other blocks")?;
        Ok(Self::new(result_types, entry_block, other_blocks))
    }
}

impl ReadFrom for section::Section<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        use section::{Section, SectionKind};

        let kind = parse_flags_value(source)?;
        let expected_length = parse_length(source).attach_printable("section byte length")?;
        let start_offset = source.file_offset();

        let section = match kind {
            SectionKind::Metadata => Section::Metadata(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::Symbol => Section::Symbol(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::Type => Section::Type(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::FunctionSignature => Section::FunctionSignature(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::FunctionInstantiation => Section::FunctionInstantiation(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::FunctionImport => Section::FunctionImport(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::FunctionDefinition => Section::FunctionDefinition(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::Code => Section::Code(parse_many_length_encoded(source)?.into_vec()),
            SectionKind::EntryPoint => Section::EntryPoint(parse_length(source).attach_printable("entry point index")?),
            SectionKind::ModuleImport => Section::ModuleImport(parse_many_length_encoded(source)?.into_vec()),
        };

        let end_offset = source.file_offset();
        let actual_length = end_offset - start_offset;

        if actual_length != expected_length {
            return Err(Error::new(end_offset)).report().attach_printable_lazy(|| format!("expected content of {kind:?} section to have a length of {expected_length} bytes, but actual length was {actual_length}"));
        }

        Ok(section)
    }
}

impl<'data> ReadFrom for module::Module<'data> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        {
            let mut magic_buffer = [0u8; crate::binary::MAGIC.len()];
            let count = source
                .read(&mut magic_buffer)
                .report()
                .change_context_lazy(|| Error::new(0))
                .attach_printable("module magic")?;

            let actual_magic = &magic_buffer[0..count];
            if actual_magic != crate::binary::MAGIC.as_slice() {
                return Err(Error::new(0)).report().attach_printable("not a valid IL4IL module file");
            }
        }

        let format_version = crate::versioning::SupportedFormat::read_from(source)?;
        let sections = parse_many_length_encoded::<section::Section<'data>, _>(source)?;
        Ok(Self::with_format_version_and_sections(format_version, sections.into_vec()))
    }
}
