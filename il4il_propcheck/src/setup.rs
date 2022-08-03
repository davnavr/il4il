//! Contains the [`Setup`] trait

use crate::generator::{Gen, Rng};

pub trait Setup: Default {
    type Rng: Rng + ?Sized;

    fn generator<'a>(&'a mut self) -> Gen<'a, Self::Rng>;
}

#[derive(Default)]
pub struct DefaultSetup(rand::rngs::ThreadRng);

impl Setup for DefaultSetup {
    type Rng = rand::rngs::ThreadRng;

    fn generator<'a>(&'a mut self) -> Gen<'a, Self::Rng> {
        Gen::new(&mut self.0, 65536)
    }
}
