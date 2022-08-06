//! Contains types that indicate whether a property test failed.

use std::borrow::Cow;
use std::fmt::{Debug, Write};

/// The result of a property test.
pub enum Assertion {
    Failure(Cow<'static, str>),
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
            Self::Failure(Cow::Owned(message))
        }
    }
}

#[macro_export]
macro_rules! assertion {
    ($e:expr) => {
        if $e {
            $crate::assertion::Assertion::Success
        } else {
            $crate::assertion::Assertion::Failure(concat!("assertion failed: ", stringify!($e)).into())
        }
    };
}

#[macro_export]
macro_rules! assertion_eq {
    ($expected:expr, $actual:expr) => {
        $crate::assertion::Assertion::are_equal($expected, $actual)
    };
}
