//! Contains types that indicate whether a property test failed.

pub enum Assertion {
    Failure(&'static str),
    Success,
}

#[macro_export]
macro_rules! assertion {
    ($e:expr) => {
        if $e {
            Assertion::Success
        } else {
            Assertion::Failure(stringify!($e))
        }
    };
}
