//! Contains the [`Arb`] trait.

use crate::generator::{Gen, Rng};

/// Trait used to generate random values.
pub trait Arb: std::fmt::Debug + 'static {
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
    u32 with U32Shrinker,
    usize with UsizeShrinker
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

    fn shrink(&self) -> Self::Shrinker {
        CharShrinker::new(*self)
    }
}

impl Arb for String {
    type Shrinker = std::iter::Empty<String>;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self {
        let maximum = gen.size();
        let count = gen.source().gen_range(0..maximum);
        (0..count).map(|_| char::arbitrary(gen)).collect()
    }

    fn shrink(&self) -> Self::Shrinker {
        std::iter::empty()
    }
}

#[derive(Debug)]
pub struct VecShrinker<T: Arb + Clone> {
    initial: Vec<T>,
    length_shrinker: UsizeShrinker,
}

impl<T: Arb + Clone> VecShrinker<T> {
    pub fn new(initial: Vec<T>) -> Self {
        Self {
            length_shrinker: UsizeShrinker::new(initial.len()),
            initial,
        }
    }
}

impl<T: Arb + Clone> Iterator for VecShrinker<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let length = self.length_shrinker.next()?;
        Some(self.initial[0..length].to_vec())
    }
}

impl<T: Arb + Clone> Arb for Vec<T> {
    type Shrinker = VecShrinker<T>;

    fn arbitrary<R: Rng + ?Sized>(gen: &mut Gen<'_, R>) -> Self {
        let length = gen.size();
        let mut items = Vec::with_capacity(length);
        for _ in 0..length {
            items.push(T::arbitrary(gen));
        }
        items
    }

    fn shrink(&self) -> Self::Shrinker {
        VecShrinker::new(self.clone())
    }
}
