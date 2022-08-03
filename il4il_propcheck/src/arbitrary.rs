//! Contains the [`Arb`] trait.

/// Trait used to generate random values.
pub trait Arb {
    fn arbitrary<R: rand::Rng + ?Sized>(gen: &mut crate::generator::Gen<'_, R>) -> Self;
}
