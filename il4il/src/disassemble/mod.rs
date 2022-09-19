//! Provides functions to print the textual representation of IL4IL modules.

mod error;
mod print;

pub use error::Error;
pub use print::*;

/// Trait for disassembly into IL4IL assembly.
pub trait Disassemble {
    fn disassemble<P: Print>(&self, output: &mut Printer<P>) -> Result;
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

/// Helper function to quickly generate a [`String`] containing IL4IL assembly.
pub fn disassembly_to_string<D: Disassemble>(d: D) -> String {
    let mut buffer = String::new();
    d.disassemble(&mut Printer::from(&mut buffer)).unwrap();
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_format_version() {
        assert_eq!(
            ".format {\n    .major 1;\n    .minor 0;\n}\n",
            disassembly_to_string(crate::versioning::Format::new(1, 0)).as_str()
        )
    }
}
