//! Module for reading, writing, and manipulating IL4IL modules.
//!
//! This module provides the [`Module`] type, which is an in-memory representation of an IL4IL module.

mod module;

pub use module::Module;

pub mod parser;
pub mod section;
pub mod writer;

/// The magic number that is the start of all IL4IL module files.
pub const MAGIC: &[u8; 6] = b"IL4IL\0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_empty_module_is_empty() {
        let builder = Module::new();
        let mut buffer = Vec::new();
        builder.write_to(&mut buffer).unwrap();

        let parsed = Module::read_from(buffer.as_slice()).unwrap();
        assert!(parsed.into_sections().is_empty());
    }
}
