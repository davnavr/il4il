//! Module to interact with the IL4IL virtual machine.

pub mod configuration;

mod module;

pub use module::Module;

use crate::loader;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

pub type Scope<'env> = std::thread::Scope<'env, 'env>;

pub struct Runtime<'env> {
    configuration: configuration::Configuration,
    modules: Mutex<Vec<Arc<Module<'env>>>>,
    scope: &'env Scope<'env>,
}

impl<'env> Runtime<'env> {
    pub fn with_configuration(configuration: configuration::Configuration, scope: &'env Scope<'env>) -> Self {
        Self {
            configuration,
            modules: Default::default(),
            scope,
        }
    }

    pub fn new(scope: &'env Scope<'env>) -> Self {
        Self::with_configuration(configuration::Configuration::HOST, scope)
    }

    pub fn configuration(&'env self) -> &'env configuration::Configuration {
        &self.configuration
    }

    pub fn scope(&'env self) -> &'env Scope {
        self.scope
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

impl Debug for Runtime<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime").field("scope", self.scope).finish_non_exhaustive()
    }
}
