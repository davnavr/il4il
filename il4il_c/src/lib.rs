//! Provides a C API for IL4IL.
//!
//! This crate is not intended to be used in Rust. When compiled to a static or dynamic library, it allows other languages (C, C++, C#,
//! Java, etc.) to use the functionality provided by [`il4il`].
//!
//! # Safety
//!
//! Note that as a C API, the provided functions make pervasive use of [`raw pointers`](prim@pointer); and as such,
//! [Rust's pointer safety rules apply](std::ptr#safety).
//!
//! To ensure that these rules are met, the [`pointer` module's validation checks](mod@pointer#safety) may result in panics.
//!
//! Additionally, almost all functions provided are **not thread safe**. This means that it is the duty of callers to do synchronization.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod browser;
pub mod error;
pub mod identifier;
pub mod metadata;
pub mod module;
pub mod pointer;
