//! Contains the [`Setup`] trait

use crate::generator::{Gen, Rng};

pub trait Setup: Default {
    type Rng: Rng + ?Sized;

    fn generator(&mut self) -> Gen<'_, Self::Rng>;
}

#[derive(Default)]
pub struct DefaultSetup(rand::rngs::ThreadRng);

impl Setup for DefaultSetup {
    type Rng = rand::rngs::ThreadRng;

    fn generator(&mut self) -> Gen<'_, Self::Rng> {
        Gen::new(&mut self.0, 65536)
    }
}
