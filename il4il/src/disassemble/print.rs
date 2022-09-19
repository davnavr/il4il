//! Contains the [`Print`] trait.

use crate::disassemble::Error;

/// Result type returned by assembly printing methods.
pub type Result = std::result::Result<(), Error>;

/// A trait for writing text into buffers or streams.
pub trait Print {
    fn print_str(&mut self, s: &str) -> Result;

    fn print_char(&mut self, c: char) -> Result {
        self.print_str(c.encode_utf8(&mut [0u8; 4]))
    }

    fn print_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result {
        if let Some(s) = args.as_str() {
            self.print_str(s)
        } else {
            self.print_str(&args.to_string())
        }
    }
}

impl<P: Print + ?Sized> Print for &mut P {
    fn print_str(&mut self, s: &str) -> Result {
        P::print_str(self, s)
    }

    fn print_char(&mut self, c: char) -> Result {
        P::print_char(self, c)
    }

    fn print_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result {
        P::print_fmt(self, args)
    }
}

impl Print for String {
    fn print_str(&mut self, s: &str) -> Result {
        self.push_str(s);
        Ok(())
    }

    fn print_char(&mut self, c: char) -> Result {
        self.push(c);
        Ok(())
    }
}

/// Allows writing text into an [`std::io::Write`].
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct IoPrint<W: std::io::Write>(W);

impl<W: std::io::Write> From<W> for IoPrint<W> {
    fn from(writer: W) -> Self {
        Self(writer)
    }
}

impl<W: std::io::Write> Print for IoPrint<W> {
    fn print_str(&mut self, s: &str) -> Result {
        self.0.write_all(s.as_bytes())?;
        Ok(())
    }

    fn print_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result {
        self.0.write_fmt(args)?;
        Ok(())
    }
}

/// Allows writing text into an [`std::fmt::Write`].
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct FmtPrint<F: std::fmt::Write>(F);

impl<F: std::fmt::Write> From<F> for FmtPrint<F> {
    fn from(writer: F) -> Self {
        Self(writer)
    }
}

impl<F: std::fmt::Write> Print for FmtPrint<F> {
    fn print_str(&mut self, s: &str) -> Result {
        self.0.write_str(s)?;
        Ok(())
    }

    fn print_char(&mut self, c: char) -> Result {
        self.0.write_char(c)?;
        Ok(())
    }

    fn print_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result {
        self.0.write_fmt(args)?;
        Ok(())
    }
}

/// A trait for conversion into a [`Print`].
pub trait IntoPrint {
    type Destination: Print;

    fn into_print(self) -> Self::Destination;
}

impl<P: Print> IntoPrint for P {
    type Destination = Self;

    fn into_print(self) -> Self::Destination {
        self
    }
}

impl IntoPrint for std::fmt::Formatter<'_> {
    type Destination = FmtPrint<Self>;

    fn into_print(self) -> Self::Destination {
        FmtPrint::from(self)
    }
}

impl IntoPrint for std::fs::File {
    type Destination = IoPrint<Self>;

    fn into_print(self) -> Self::Destination {
        IoPrint::from(self)
    }
}

impl IntoPrint for std::io::Stdout {
    type Destination = IoPrint<Self>;

    fn into_print(self) -> Self::Destination {
        IoPrint::from(self)
    }
}

/// Indicates the character to use for indentation. The cause of many internet flame wars.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum IndentationCharacter {
    Tab,
    Space,
}

impl Default for IndentationCharacter {
    fn default() -> Self {
        Self::Space
    }
}

/// Indicates the indentation to use when printing IL4IL assembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Indentation {
    character: IndentationCharacter,
    size: std::num::NonZeroU8,
}

impl Indentation {
    fn print<P: Print>(&self, destination: &mut P) -> Result {
        let size = usize::from(self.size.get());
        match self.character {
            IndentationCharacter::Tab => std::iter::repeat('\t').take(size).try_for_each(|c| destination.print_char(c)),
            IndentationCharacter::Space => match size {
                2 => destination.print_str("  "),
                4 => destination.print_str("    "),
                _ => std::iter::repeat(' ').take(size).try_for_each(|c| destination.print_char(c)),
            },
        }
    }
}

impl std::fmt::Display for Indentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.print(&mut FmtPrint::from(f)).map_err(|_| Default::default())
    }
}

impl Indentation {
    pub const ONE_TAB: Self = Self::new(IndentationCharacter::Tab, unsafe {
        // Safety: 1 != 0
        std::num::NonZeroU8::new_unchecked(1)
    });

    pub const FOUR_SPACES: Self = Self::new(IndentationCharacter::Space, unsafe {
        // Safety: 4 != 0
        std::num::NonZeroU8::new_unchecked(4)
    });

    pub const fn new(character: IndentationCharacter, size: std::num::NonZeroU8) -> Self {
        Self { character, size }
    }

    pub const fn character(&self) -> IndentationCharacter {
        self.character
    }

    /// The number of characters to use in indentation.
    pub fn size(&self) -> usize {
        self.size.get().into()
    }
}

impl Default for Indentation {
    fn default() -> Self {
        Self::FOUR_SPACES
    }
}

/// Options that determine what the resulting IL4IL assembly output will look like.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PrintOptions {
    pub indentation: Indentation,
}

impl PrintOptions {
    pub const DEFAULT: Self = Self {
        indentation: Indentation::FOUR_SPACES,
    };
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug)]
pub struct Printer<'a, P: Print> {
    destination: P,
    options: &'a PrintOptions,
    indentation_level: u8,
    is_new_line: bool,
}

impl<'a, P: Print> Printer<'a, P> {
    pub fn with_options<D: IntoPrint<Destination = P>>(destination: D, options: &'a PrintOptions) -> Self {
        Self {
            destination: destination.into_print(),
            options,
            indentation_level: 0,
            is_new_line: true,
        }
    }

    pub fn new<D: IntoPrint<Destination = P>>(destination: D) -> Self {
        Self::with_options(destination, &PrintOptions::DEFAULT)
    }

    pub fn options(&self) -> &PrintOptions {
        self.options
    }

    fn print_new_line(&mut self) -> Result {
        if !self.is_new_line {
            self.is_new_line = true;
            self.destination.print_char('\n')?;
        }

        Ok(())
    }

    fn flush_indentation(&mut self) -> Result {
        if self.is_new_line {
            self.is_new_line = false;
            for _ in 0..self.indentation_level {
                self.options.indentation.print(&mut self.destination)?;
            }
        }

        Ok(())
    }

    fn indent(&mut self) {
        self.indentation_level += 1;
    }

    fn dedent(&mut self) {
        self.indentation_level -= 1;
    }

    fn print_start(&mut self) -> Result {
        self.print_new_line()?;
        self.flush_indentation()
    }

    pub(super) fn print_directive<'b>(&'b mut self, name: &str) -> PrintContent<'a, 'b, P> {
        PrintContent(PrintHelper {
            result: self.print_start().and_then(|_| {
                self.destination.print_char('.')?;
                self.destination.print_str(name)
            }),
            printer: self,
        })
    }
}

impl<P: IntoPrint> From<P> for Printer<'_, P::Destination> {
    fn from(source: P) -> Self {
        Self::new(source)
    }
}

struct PrintHelper<'a, 'b, P: Print> {
    printer: &'b mut Printer<'a, P>,
    result: Result,
}

#[repr(transparent)]
pub(super) struct PrintContent<'a, 'b, P: Print>(PrintHelper<'a, 'b, P>);

impl<'a, 'b, P: Print> PrintContent<'a, 'b, P> {
    pub fn with_attributes<F: FnOnce(&mut PrintAttributes<'_, '_, P>) -> Result>(&mut self, f: F) -> &mut Self {
        self.0.result = std::mem::replace(&mut self.0.result, Ok(())).and_then(|_| {
            let mut attributes = PrintAttributes(self.0.printer);
            f(&mut attributes)
        });
        self
    }

    pub fn block(&mut self) -> PrintBlock<'a, '_, P> {
        PrintBlock(PrintHelper {
            result: std::mem::replace(&mut self.0.result, Ok(())).and_then(|_| {
                self.0.printer.destination.print_str(" {")?;
                self.0.printer.print_new_line()?;
                self.0.printer.indent();
                Ok(())
            }),
            printer: self.0.printer,
        })
    }

    pub fn finish(&mut self) -> Result {
        std::mem::replace(&mut self.0.result, Ok(()))?;
        self.0.printer.destination.print_char(';')?;
        self.0.printer.print_new_line()
    }
}

#[repr(transparent)]
pub(super) struct PrintBlock<'a, 'b, P: Print>(PrintHelper<'a, 'b, P>);

impl<'a, 'b, P: Print> PrintBlock<'a, 'b, P> {
    pub fn with_printer<F: FnOnce(&mut Printer<'a, P>) -> Result>(&mut self, f: F) -> &mut Self {
        self.0.result = std::mem::replace(&mut self.0.result, Ok(())).and_then(|_| f(self.0.printer));
        self
    }

    pub fn finish(&mut self) -> Result {
        std::mem::replace(&mut self.0.result, Ok(()))?;
        self.0.printer.dedent();
        self.0.printer.print_new_line()?;
        self.0.printer.destination.print_char('}')?;
        self.0.printer.print_new_line()
    }
}

#[repr(transparent)]
pub(super) struct PrintAttributes<'a, 'b, P: Print>(&'b mut Printer<'a, P>);

impl<P: Print> PrintAttributes<'_, '_, P> {
    pub(super) fn with_print<F: FnOnce(&mut P) -> Result>(&mut self, f: F) -> Result {
        self.0.destination.print_char(' ')?;
        f(&mut self.0.destination)
    }

    pub fn print_display<D: std::fmt::Display>(&mut self, item: D) -> Result {
        self.with_print(|destination| destination.print_fmt(format_args!("{item}")))
    }
}
