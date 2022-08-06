//! Contains the [`Run`] trait.

use crate::generator::{Gen, Rng};
use crate::{Arb, Assertion};
use std::fmt::Write;

pub use crate::assertion::Failure;

/// Indicates that a test did not fail.
#[derive(Clone, Debug)]
pub enum NonFailure {
    Skipped,
    Success,
}

/// Represents a test that has been shrunk.
pub trait ShrunkTest {
    fn run(self, inputs: &mut String) -> Result<NonFailure, Failure>;
}

impl<F: FnOnce(&mut String) -> Result<NonFailure, Failure>> ShrunkTest for F {
    fn run(self, inputs: &mut String) -> Result<NonFailure, Failure> {
        (self)(inputs)
    }
}

pub type TestShrinker = Box<dyn Iterator<Item = Box<dyn ShrunkTest>>>;

pub type PropertyResult = Result<NonFailure, (TestShrinker, Failure)>;

/// Represents a property test.
pub trait PropertyTest: 'static {
    fn test<R: ?Sized + Rng>(self, inputs: &mut String, gen: &mut Gen<'_, R>) -> PropertyResult;
}

fn assertion_to_result(assertion: Option<Assertion>) -> Result<NonFailure, Failure> {
    match assertion {
        Some(Assertion::Success) => Ok(NonFailure::Success),
        None => Ok(NonFailure::Skipped),
        Some(Assertion::Failure(message)) => Err(message),
    }
}

fn handle_assertion(assertion: Option<Assertion>, shrinker: impl FnOnce() -> TestShrinker) -> PropertyResult {
    match assertion {
        Some(Assertion::Success) => Ok(NonFailure::Success),
        None => Ok(NonFailure::Skipped),
        Some(Assertion::Failure(message)) => Err((shrinker(), message)),
    }
}

impl<A: Arb> PropertyTest for fn(A) -> Option<Assertion> {
    fn test<R: ?Sized + Rng>(self, inputs: &mut String, gen: &mut Gen<'_, R>) -> PropertyResult {
        let a = A::arbitrary(gen);
        let shrinker = a.shrink();
        write!(inputs, "{:?}", a).unwrap();
        handle_assertion(self(a), move || {
            Box::from(shrinker.map(move |item| {
                Box::new(move |inputs: &mut String| {
                    write!(inputs, "{:?}", item).unwrap();
                    assertion_to_result(self(item))
                }) as Box<dyn ShrunkTest>
            }))
        })
    }
}

// impl<A: Arb, B: Arb> Test for fn(A, B) -> Option<Assertion> {
//     fn test<R: ?Sized + Rng>(&self, inputs: &mut String, gen: &mut Gen<'_, R>) -> Option<Assertion> {
//         let a = A::arbitrary(gen);
//         let b = B::arbitrary(gen);
//         write!(inputs, "{:?}, {:?}", a, b).unwrap();
//         self(a, b)
//     }
// }
