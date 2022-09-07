//! Library for loading IL4IL modules and their imports.
//!
//! Lazy initialization is used extensively in order to ensure that allocations only occur when necessary.

mod debug;

pub mod code;
pub mod environment;
pub mod function;
pub mod module;
pub mod types;
