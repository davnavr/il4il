//! Module for assembler errors.

use crate::location::Location;

/// Represents an error encountered while parsing or assembling an IL4IL module.
#[derive(Debug)]
pub struct Error {
    location: Location,
}

impl Error {
    pub fn location(&self) -> &Location {
        &self.location
    }
}
