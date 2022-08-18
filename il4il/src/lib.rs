//! Provides a reader, writer, and validator for IL4IL modules.

#![deny(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

pub mod binary;
pub mod function;
pub mod identifier;
pub mod index;
pub mod integer;
pub mod module;
pub mod type_system;
pub mod validation;
pub mod versioning;

#[cfg(test)]
use il4il_propcheck as propcheck;
