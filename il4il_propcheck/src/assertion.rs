//! Contains types that indicate whether a property test failed.

use std::fmt::{Debug, Write};

pub type Message = std::borrow::Cow<'static, str>;

/// The result of a property test.
pub enum Assertion {
    Failure(Message),
    Success,
}

impl Assertion {
    pub fn are_equal<A, B>(expected: A, actual: B) -> Self
    where
        A: Debug,
        B: Debug,
        A: PartialEq<B>,
    {
        if expected == actual {
            Self::Success
        } else {
            let mut message = String::new();
            write!(&mut message, "Expected:\n{:?}\nActual:\n{:?}\n", expected, actual).unwrap();
            Self::Failure(message.into())
        }
    }
}
