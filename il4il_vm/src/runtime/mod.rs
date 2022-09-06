//! Module to interact with the IL4IL virtual machine.

pub mod configuration;

mod function;
mod module;

pub use function::{Function, HostFunction, HostFunctionResult};
pub use module::Module;

use crate::loader;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Runtime<'env> {
    configuration: configuration::Configuration,
    modules: Mutex<Vec<Arc<Module<'env>>>>,
}

impl<'env> Runtime<'env> {
    pub fn with_configuration(configuration: configuration::Configuration) -> Self {
        Self {
            configuration,
            modules: Default::default(),
        }
    }

    pub fn new() -> Self {
        Self::with_configuration(configuration::Configuration::HOST)
    }

    pub fn configuration(&'env self) -> &'env configuration::Configuration {
        &self.configuration
    }

    pub fn load_module(&'env self, module: il4il::validation::ValidModule<'env>) -> Arc<Module<'env>> {
        let loaded = Arc::new(Module::new(
            self,
            loader::module::Module::from_valid_module(module, &self.configuration.loader_context),
        ));
        self.modules.lock().unwrap().push(loaded.clone());
        loaded
    }
}

impl Default for Runtime<'_> {
    fn default() -> Self {
        Self::new()
    }
}
