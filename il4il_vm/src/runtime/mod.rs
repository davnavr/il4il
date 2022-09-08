//! Module to interact with the IL4IL virtual machine.

pub mod configuration;

mod function;
mod module;

pub use function::{Function, HostFunction, HostFunctionResult};
pub use module::Module;

pub mod resolver;

use crate::loader;
use std::sync::{Arc, Mutex};

pub struct Runtime<'env> {
    configuration: configuration::Configuration,
    default_resolver: resolver::BoxedResolver,
    modules: Mutex<Vec<Arc<Module<'env>>>>,
}

impl<'env> Runtime<'env> {
    pub fn with_configuration_and_resolver(configuration: configuration::Configuration, resolver: resolver::BoxedResolver) -> Self {
        Self {
            configuration,
            default_resolver: resolver,
            modules: Default::default(),
        }
    }
    pub fn with_configuration(configuration: configuration::Configuration) -> Self {
        Self::with_configuration_and_resolver(configuration, Box::<()>::default())
    }

    pub fn new() -> Self {
        Self::with_configuration(configuration::Configuration::HOST)
    }

    pub fn configuration(&'env self) -> &'env configuration::Configuration {
        &self.configuration
    }

    pub fn default_resolver(&'env self) -> &'env dyn resolver::Resolver {
        self.default_resolver.as_ref()
    }

    pub fn load_module(
        &'env self,
        module: il4il::validation::ValidModule<'env>,
        resolver: Option<Box<dyn resolver::Resolver>>,
    ) -> Arc<Module<'env>> {
        let loaded = Arc::new(Module::new(
            self,
            loader::module::Module::from_valid_module(module, &self.configuration.loader_context),
            resolver,
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

impl std::fmt::Debug for Runtime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("configuration", &self.configuration)
            .field("modules", &self.modules)
            .finish_non_exhaustive()
    }
}
