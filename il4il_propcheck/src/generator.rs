//! Module to generate random values.

pub use rand::Rng;

#[derive(Debug)]
pub struct Gen<'a, R: ?Sized> {
    source: &'a mut R,
    size: usize,
}

impl<'a, R: Rng + ?Sized> Gen<'a, R> {
    pub fn new(source: &'a mut R, size: usize) -> Self {
        Self { source, size }
    }
}

impl<R: Rng + ?Sized> Gen<'_, R> {
    pub fn source(&mut self) -> &mut R {
        self.source
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
