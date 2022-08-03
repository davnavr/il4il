//! Provides a reader, writer, and validator for IL4IL modules.

#![deny(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

pub mod binary;
pub mod identifier;
pub mod integer;
pub mod versioning;
