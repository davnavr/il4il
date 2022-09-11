//! Module for assembler errors.

use crate::location::Location;
use std::fmt::{Formatter, Write};
use std::ops::Range;

type Message = dyn Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result;

/// Represents an error encountered while parsing or assembling an IL4IL module.
pub struct Error {
    location: Range<Location>,
    message: Box<Message>,
}

impl Error {
    pub fn new<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result + 'static>(location: Range<Location>, message: F) -> Self {
        Self {
            location,
            message: Box::new(message),
        }
    }

    pub fn location(&self) -> &Range<Location> {
        &self.location
    }

    pub fn format_message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.message)(f)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[repr(transparent)]
        struct MessageDebug<'a>(&'a Message);

        impl std::fmt::Debug for MessageDebug<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_char('\'')?;
                (self.0)(f)?;
                f.write_char('\'')
            }
        }

        f.debug_struct("Error")
            .field("location", &self.location)
            .field("message", &MessageDebug(&self.message))
            .finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} - ", self.location.start.line, self.location.start.column)?;
        (self.message)(f)
    }
}

#[cfg(test)]
pub(crate) fn assert_ok<'a, E>(errors: E)
where
    E: IntoIterator<Item = &'a Error>,
    E::IntoIter: std::iter::ExactSizeIterator + Clone,
{
    let iter_errors = errors.into_iter();
    let count = iter_errors.len();
    if count > 0 {
        struct DisplayErrors<I>(I);

        impl<'a, I: Iterator<Item = &'a Error> + Clone> std::fmt::Display for DisplayErrors<I> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                for e in self.0.clone() {
                    writeln!(f, "{}", e)?;
                }
                Ok(())
            }
        }

        panic!("failed with {count} errors:\n{}", DisplayErrors(iter_errors));
    }
}
