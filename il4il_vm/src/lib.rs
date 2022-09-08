//! Provides an interpreter for the IL4IL virtual machine.

pub use il4il as model;
pub use il4il_loader as loader;

pub mod host;
pub mod interpreter;
pub mod runtime;
