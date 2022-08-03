//! Contains the [`Arb`] trait.

use crate::generator::{Gen, Rng};

pub trait Comparable {
    fn is_smaller_than(&self, other: &Self) -> bool;
}

impl<T: ?Sized + PartialOrd> Comparable for T {
    fn is_smaller_than(&self, other: &Self) -> bool {
        self < other
    }
}

/// Trait used to generate random values.
pub trait Arb: std::fmt::Debug + Clone + Comparable {
    type Shrinker: Iterator<Item = Self>;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self;

    fn shrink(self) -> Self::Shrinker;
}

impl Arb for () {
    type Shrinker = std::iter::Empty<()>;

    fn arbitrary<R: Rng + ?Sized>(_: &mut Gen<'_, R>) -> Self {}

    fn shrink(self) -> Self::Shrinker {
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

                fn shrink(self) -> Self::Shrinker {
                    Self::Shrinker::new(self)
                }
            }
        )*
    };
}

unsigned_integer_arb! {
    u32 with U32Shrinker
}

#[derive(Clone, Debug)]
pub struct CharShrinker(U32Shrinker);

impl CharShrinker {
    pub fn new(initial: char) -> Self {
        Self(U32Shrinker::new(initial.into()))
    }
}

impl Iterator for CharShrinker {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        for value in self.0.by_ref() {
            if let Some(c) = char::from_u32(value) {
                return Some(c);
            }
        }

        None
    }
}

impl Arb for char {
    type Shrinker = CharShrinker;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self {
        if gen.source().gen_bool(0.66) {
            gen.source().gen_range(' '..'~')
        } else {
            gen.source().gen()
        }
    }

    fn shrink(self) -> Self::Shrinker {
        CharShrinker::new(self)
    }
}

impl Arb for String {
    type Shrinker = std::iter::Empty<String>;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self {
        let maximum = gen.size();
        let count = gen.source().gen_range(0..maximum);
        (0..count).map(|_| char::arbitrary(gen)).collect()
    }

    fn shrink(self) -> Self::Shrinker {
        std::iter::empty()
    }
}
