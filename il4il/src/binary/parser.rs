//! Module for parsing the contents of an IL4IL module.

use crate::function;
use crate::identifier::Identifier;
use crate::index;
use crate::integer;
use crate::module::section::{self, Section};
use crate::type_system;
use std::borrow::Cow;
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
    fn push_location(&mut self, name: &'static str) {
        self.locations.push(Location {
            name,
            starting_file_offset: self.file_offset,
            offset: 0,
        })
    }

    /// Pops a location from the top of the location stack, returning a file offset to the start of the location.
    fn pop_location(&mut self) -> usize {
        let location = self.locations.pop().unwrap();
        self.file_offset - location.starting_file_offset
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

/// Error type used when an unsigned integer length or index cannot be converted to a [`usize`].
#[derive(Clone, Debug, thiserror::Error)]
#[error("{0} is too large for the current platform")]
pub struct SizeConversionError(integer::VarU28);

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
}

/// Error type used when some combination of flags is invalid.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{value} is not a valid {name}")]
pub struct InvalidFlagsError {
    name: &'static str,
    value: String,
}

impl InvalidFlagsError {
    fn new<T: std::fmt::UpperHex>(name: &'static str, value: T) -> Self {
        Self {
            name,
            value: format!("{value:#02X}"),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("expected {:?} section to have a length of {expected} bytes, but {actual} bytes were parsed")]
pub struct SectionLengthError {
    section: section::SectionKind,
    expected: usize,
    actual: usize,
}

/// Error type used when a type is not valid for a reason other than being invalid (e.g. a float type was used when an integer type was
/// expected)
#[derive(Clone, Debug, thiserror::Error)]
pub struct UnsupportedTypeError {
    type_reference: type_system::Reference,
    context: &'static str,
}

impl Display for UnsupportedTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported type {}, {}", self.type_reference, self.context)
    }
}

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
    UnsupportedIntegerLength(#[from] integer::LengthError),
    #[error(transparent)]
    SizeConversion(#[from] SizeConversionError),
    #[error(transparent)]
    InvalidFlags(#[from] InvalidFlagsError),
    #[error(transparent)]
    InvalidIdentifier(#[from] crate::identifier::ParseError),
    #[error(transparent)]
    SectionLengthMismatch(#[from] SectionLengthError),
    #[error(transparent)]
    InvalidTypeTag(#[from] type_system::InvalidTagError),
    #[error(transparent)]
    UnsupportedTypeKind(#[from] UnsupportedTypeError),
    #[error(transparent)]
    InvalidIntegerBitWidth(#[from] type_system::InvalidBitWidthError),
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

impl ReadFrom for integer::VarU28 {
    fn read_from<R: Read>(mut source: &mut Source<R>) -> Result<Self> {
        match Self::read_from(&mut source) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(source.create_error(error)),
            Err(error) => Err(source.create_error(error)),
        }
    }
}

impl ReadFrom for integer::VarI28 {
    fn read_from<R: Read>(mut source: &mut Source<R>) -> Result<Self> {
        match Self::read_from(&mut source) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(source.create_error(error)),
            Err(error) => Err(source.create_error(error)),
        }
    }
}

fn parse_length<L: From<usize>>(src: &mut Source<impl Read>) -> Result<L> {
    src.save_file_offset();
    let value = <integer::VarU28 as ReadFrom>::read_from(src)?;
    usize::try_from(value)
        .map(L::from)
        .map_err(|_| src.create_error(SizeConversionError(value)))
}

fn parse_many_length_encoded<T: ReadFrom, R: Read>(src: &mut Source<R>) -> Result<Box<[T]>> {
    let count = parse_length(src)?;
    T::read_many(src, count)
}

fn parse_flags_value<T, R>(src: &mut Source<R>) -> Result<T>
where
    T: FlagsValue,
    T::Value: ReadFrom,
    R: Read,
{
    src.save_file_offset();
    let flags = <T::Value>::read_from(src)?;
    T::from_value(flags).ok_or_else(|| src.create_error(InvalidFlagsError::new(T::name(), flags)))
}

impl ReadFrom for Identifier {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        Self::from_utf8(parse_many_length_encoded::<u8, _>(source)?.into_vec()).map_err(|e| source.create_error(e))
    }
}

impl ReadFrom for section::Metadata<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        source.push_location("metadata");
        let metadata = match parse_flags_value(source)? {
            section::MetadataKind::Name => section::Metadata::Name(Cow::Owned(Identifier::read_from(source)?)),
        };
        source.pop_location();
        Ok(metadata)
    }
}

impl ReadFrom for type_system::IntegerSize {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        Self::from_u28(<integer::VarU28 as ReadFrom>::read_from(source)?).map_err(|e| source.create_error(e))
    }
}

impl ReadFrom for type_system::Reference {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        let tag_value = <integer::VarI28 as ReadFrom>::read_from(source)?;
        match integer::VarU28::try_from(tag_value) {
            Ok(index) => Ok(type_system::Reference::Index(index::Type::new(
                usize::try_from(index).map_err(|_| source.create_error(SizeConversionError(index)))?,
            ))),
            Err(_) => {
                let value = type_system::TypeTag::try_from(tag_value).map_err(|e| source.create_error(e))?;
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
        match type_system::Reference::read_from(source)? {
            type_system::Reference::Inline(ty) => Ok(ty),
            type_reference => Err(source.create_error(UnsupportedTypeError {
                type_reference,
                context: "type index is not allowed here",
            })),
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

impl ReadFrom for Section<'_> {
    fn read_from<R: Read>(source: &mut Source<R>) -> Result<Self> {
        source.push_location("section");
        let expected_length = parse_length(source)?;
        let start_offset = source.file_offset();

        let kind = parse_flags_value(source)?;
        let section = match kind {
            section::SectionKind::Metadata => Section::Metadata(parse_many_length_encoded(source)?.into_vec()),
            section::SectionKind::Type => Section::Type(parse_many_length_encoded(source)?.into_vec()),
            section::SectionKind::FunctionSignature => Section::FunctionSignature(parse_many_length_encoded(source)?.into_vec()),
            #[allow(unreachable_patterns)]
            _ => todo!(),
        };

        let end_offset = source.pop_location();
        let actual_length = end_offset - start_offset;

        if actual_length != expected_length {
            return Err(source.create_error(SectionLengthError {
                section: kind,
                expected: expected_length,
                actual: actual_length,
            }));
        }

        Ok(section)
    }
}

impl<'data> ReadFrom for crate::module::Module<'data> {
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
        let sections = parse_many_length_encoded::<Section<'data>, _>(source)?;
        source.pop_location();

        Ok(Self::with_format_version_and_sections(format_version, sections.into_vec()))
    }
}
