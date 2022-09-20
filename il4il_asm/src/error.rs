//! Module for assembler errors.

use crate::location::Location;
use std::fmt::{Formatter, Write};
use std::ops::Range;

/// Trait for error messages.
pub(crate) trait Message: 'static {
    fn message(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
}

impl Message for Box<dyn Message> {
    fn message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let m: &dyn Message = self.as_ref();
        m.message(f)
    }
}

impl Message for &'static str {
    fn message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl Message for String {
    fn message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<F: Fn(&mut Formatter<'_>) -> std::fmt::Result + 'static> Message for F {
    fn message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self)(f)
    }
}

/// Represents an error encountered while parsing or assembling an IL4IL module.
#[must_use]
pub struct Error {
    location: Range<Location>,
    message: Box<dyn Message>,
}

impl Error {
    pub(crate) fn new<M: Message>(location: Range<Location>, message: M) -> Self {
        Self {
            location,
            message: Box::new(message),
        }
    }

    pub fn location(&self) -> &Range<Location> {
        &self.location
    }

    pub fn format_message(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.message.message(f)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[repr(transparent)]
        struct MessageDebug<'a>(&'a dyn Message);

        impl std::fmt::Debug for MessageDebug<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_char('\'')?;
                self.0.message(f)?;
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
        self.message.message(f)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

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
