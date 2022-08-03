//! Contains the [`Arb`] trait.

use crate::generator::{Gen, Rng};

/// Trait used to generate random values.
pub trait Arb: std::fmt::Debug + Clone {
    type Shrinker: Iterator<Item = Self>;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self;

    fn shrink(&self) -> Self::Shrinker;
}

impl Arb for () {
    type Shrinker = std::iter::Empty<()>;

    fn arbitrary<R: Rng + ?Sized>(_: &mut Gen<'_, R>) -> Self {}

    fn shrink(&self) -> Self::Shrinker {
        std::iter::empty()
    }
}
