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
