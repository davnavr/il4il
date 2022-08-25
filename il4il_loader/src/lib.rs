//! Library for loading IL4IL modules and their imports.
//!
//! Lazy initialization is used extensively in order to ensure that allocations occur only when necessary.

mod debug;

pub mod code;
pub mod environment;
pub mod function;
pub mod module;
