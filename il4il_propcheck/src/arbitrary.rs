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

macro_rules! unsigned_integer_arb {
    ($($ty:tt with $shrinker_name:ident),*) => {
        $(
            #[derive(Clone, Debug)]
            pub struct $shrinker_name {
                current: $ty,
                halved: $ty,
            }

            impl $shrinker_name {
                pub fn new(initial: $ty) -> Self {
                    Self {
                        current: initial,
                        halved: initial / 2,
                    }
                }
            }

            impl Iterator for $shrinker_name {
                type Item = $ty;

                fn next(&mut self) -> Option<$ty> {
                    let shrunk = self.current - self.halved;
                    if shrunk < self.current {
                        self.halved /= 2;
                        Some(shrunk)
                    } else {
                        None
                    }
                }
            }

            impl Arb for $ty {
                type Shrinker = $shrinker_name;

                fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self {
                    gen.source().gen()
                }

                fn shrink(&self) -> Self::Shrinker {
                    Self::Shrinker::new(*self)
                }
            }
        )*
    };
}

unsigned_integer_arb! {
    u32 with U32Shrinker
}
