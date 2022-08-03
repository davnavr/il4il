//! Module for reading, writing, and manipulating IL4IL modules.
//!
//! This module provides the [`Module`] type, which is an in-memory representation of an IL4IL module.

mod module;

pub use module::Module;

pub mod section;
