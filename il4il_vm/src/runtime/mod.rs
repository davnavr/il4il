//! Module to interact with the IL4IL virtual machine.

pub mod configuration;

pub struct Runtime<'env> {
    configuration: configuration::Configuration,
    modules: elsa::FrozenVec<&'env ()>,
}

impl<'env> Runtime<'env> {
    pub fn configuration(&'env self) -> &'env configuration::Configuration {
        &self.configuration
    }
}
