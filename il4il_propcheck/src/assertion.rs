//! Contains types that indicate whether a property test failed.

pub enum Assertion {
    Failure(&'static str),
    Success,
}

#[macro_export]
macro_rules! assertion {
    ($e:expr) => {
        if $e {
            $crate::assertion::Assertion::Success
        } else {
            $crate::assertion::Assertion::Failure(concat!("assertion failed: ", stringify!($e)))
        }
    };
}
