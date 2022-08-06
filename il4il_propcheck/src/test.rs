//! Contains the [`Run`] trait.

use crate::assertion::Message;
use crate::generator::{Gen, Rng};
use crate::{Arb, Assertion};
use std::fmt::Write;

/// Indicates that a test failed.
pub enum Failure {
    Message(Message),
    Panic(Box<dyn std::any::Any + Send + 'static>),
}

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

fn shrunk_test<F: FnOnce(&mut String) -> Result<NonFailure, Failure> + 'static>(test: F) -> Box<dyn ShrunkTest> {
    Box::new(test) as Box<dyn ShrunkTest>
}

fn wrap_shrunk_test<F: FnOnce() -> Option<Assertion>>(test: F) -> Result<NonFailure, Failure> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
    match result {
        Ok(Some(Assertion::Success)) => Ok(NonFailure::Success),
        Ok(None) => Ok(NonFailure::Skipped),
        Ok(Some(Assertion::Failure(message))) => Err(Failure::Message(message)),
        Err(info) => Err(Failure::Panic(info)),
    }
}

fn wrap_property_test<F: FnOnce() -> Option<Assertion>>(test: F, shrinker: impl FnOnce() -> TestShrinker) -> PropertyResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
    match result {
        Ok(Some(Assertion::Success)) => Ok(NonFailure::Success),
        Ok(None) => Ok(NonFailure::Skipped),
        Ok(Some(Assertion::Failure(message))) => Err((shrinker(), Failure::Message(message))),
        Err(info) => Err((shrinker(), Failure::Panic(info)))
    }
}

impl<A: Arb> PropertyTest for fn(A) -> Option<Assertion> {
    fn test<R: ?Sized + Rng>(self, inputs: &mut String, gen: &mut Gen<'_, R>) -> PropertyResult {
        let a = A::arbitrary(gen);
        let shrinker = a.shrink();
        write!(inputs, "{:?}", a).unwrap();
        wrap_property_test(|| self(a), move || {
            Box::from(shrinker.map(move |item| {
                shrunk_test(move |inputs: &mut String| {
                    write!(inputs, "{:?}", item).unwrap();
                    wrap_shrunk_test(|| self(item))
                })
            }))
        })
    }
}

impl<A: Arb, B: Arb> PropertyTest for fn(A, B) -> Option<Assertion> {
    fn test<R: ?Sized + Rng>(self, inputs: &mut String, gen: &mut Gen<'_, R>) -> PropertyResult {
        let a = A::arbitrary(gen);
        let b = B::arbitrary(gen);
        let shrinker_a = a.shrink();
        let shrinker_b = b.shrink();
        write!(inputs, "{:?}, {:?}", a, b).unwrap();
        wrap_property_test(|| self(a, b), move || {
            Box::from(shrinker_a.zip(shrinker_b).map(move |(item_a, item_b)| {
                shrunk_test(move |inputs: &mut String| {
                    write!(inputs, "{:?}, {:?}", item_a, item_b).unwrap();
                    wrap_shrunk_test(|| self(item_a, item_b))
                })
            }))
        })
    }
}
