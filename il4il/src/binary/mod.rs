//! Module for reading and writing IL4IL modules.
//!
//! The [`parser`] and [`writer`] modules contain traits and low-level routines for the reading and writing of module contents
//! respectively.
//!
//! For an easy way to simply read and write modules, the [`Module`] struct provides the [`Module::read_from`] and
//! [`Module::write_to`] functions.
//!
//! [`Module`]: crate::module::Module
//! [`Module::read_from`]: crate::module::Module::read_from
//! [`Module::write_to`]: crate::module::Module::write_to

pub mod parser;
pub mod writer;

/// The magic number that is the start of all IL4IL module files.
pub const MAGIC: &[u8; 6] = b"IL4IL\0";

#[cfg(test)]
mod tests {
    use crate::module::Module;

    #[test]
    fn parsed_empty_module_is_empty() {
        let builder = Module::new();
        let mut buffer = Vec::new();
        builder.write_to(&mut buffer).unwrap();

        let parsed = Module::read_from(buffer.as_slice()).unwrap();
        assert!(parsed.into_sections().is_empty());
    }
}
