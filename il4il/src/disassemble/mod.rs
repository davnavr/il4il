//! Provides functions to print the textual representation of IL4IL modules.

mod error;
mod print;

pub use error::Error;
pub use print::*;

/// Trait for disassembly into IL4IL assembly.
pub trait Disassemble {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result;
}

fn disassemble_many<'a, D: Disassemble + 'a, I: IntoIterator<Item = &'a D>, P: Print>(items: I, output: &mut Printer<P>) -> Result {
    items.into_iter().try_for_each(|i| i.disassemble(output))
}

impl Disassemble for crate::versioning::Format {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result {
        output
            .print_directive("format")
            .block()
            .with_printer(|p| {
                p.print_directive("major")
                    .with_attributes(|a| a.print_display(self.major))
                    .finish()?;
                p.print_directive("minor").with_attributes(|a| a.print_display(self.minor)).finish()
            })
            .finish()
    }
}

impl Disassemble for crate::versioning::SupportedFormat {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result {
        self.version().disassemble(output)
    }
}

impl Disassemble for crate::module::section::Metadata<'_> {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result {
        match self {
            Self::Name(name) => output
                .print_directive("name")
                .with_attributes(|a| a.with_print(|p| p.print_fmt(format_args!("{:?}", name.name))))
                .finish(),
        }
    }
}

impl Disassemble for crate::module::section::Section<'_> {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result {
        use crate::module::section::SectionKind;

        output
            .print_directive("section")
            .with_attributes(|a| {
                a.print_display(match self.kind() {
                    SectionKind::Metadata => "metadata",
                    SectionKind::Symbol => "symbol",
                    SectionKind::Type => "type",
                    _ => "TODO",
                })
            })
            .block()
            .with_printer(|p| match self {
                Self::Metadata(metadata) => disassemble_many(metadata.iter(), p),
                _ => todo!(),
            })
            .finish()
    }
}

impl Disassemble for crate::module::Module<'_> {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result {
        self.format_version().disassemble(output)?;
        disassemble_many(self.sections().iter(), output)
    }
}

/// Helper function to quickly generate a [`String`] containing IL4IL assembly.
pub fn disassembly_to_string<D: Disassemble>(d: D) -> String {
    let mut buffer = String::new();
    d.disassemble(&mut Printer::from(&mut buffer)).unwrap();
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::section::{self, Section};
    use crate::module::{self, Module};

    #[test]
    fn write_format_version() {
        assert_eq!(
            ".format {\n    .major 1;\n    .minor 0;\n}",
            disassembly_to_string(crate::versioning::Format::new(1, 0)).as_str()
        )
    }

    #[test]
    fn very_simple_module() {
        let mut module = Module::new();
        module
            .sections_mut()
            .push(Section::Metadata(vec![section::Metadata::Name(module::ModuleName::from_name(
                crate::identifier::Id::new("Hello").unwrap(),
            ))]));

        assert_eq!(
            ".format {\n    .major 0;\n    .minor 1;\n}\n.section metadata {\n    .name \"Hello\";\n}",
            disassembly_to_string(module)
        )
    }
}
