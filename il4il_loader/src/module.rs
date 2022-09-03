//! Contains the [`Module`] struct.

use crate::code;
use crate::debug::LazyDebug;
use crate::environment::Context;
use crate::function;
use crate::types;
use std::fmt::{Debug, Formatter};

type Types<'env> = lazy_init::LazyTransform<Vec<il4il::type_system::Type>, Box<[types::Type<'env>]>>;

type FunctionDefinitions<'env> = lazy_init::LazyTransform<Vec<il4il::function::Definition>, Box<[function::template::Definition<'env>]>>;

type FunctionTemplates<'env> = lazy_init::LazyTransform<il4il::function::TemplateLookup, Box<[function::template::Template<'env>]>>;

type FunctionInstantiations<'env> = lazy_init::LazyTransform<Vec<il4il::function::Instantiation>, Box<[function::Instantiation<'env>]>>;

type FunctionBodies<'env> = lazy_init::LazyTransform<Vec<il4il::function::Body>, Box<[code::Code<'env>]>>;

type EntryPoint<'env> = lazy_init::LazyTransform<Option<il4il::index::FunctionInstantiation>, Option<&'env function::Instantiation<'env>>>;

/// Encapsulates an IL4IL module and its associated state, allowing for easy resolution of imports, types, etc.
pub struct Module<'env> {
    environment: &'env Context,
    types: Types<'env>,
    function_definitions: FunctionDefinitions<'env>,
    function_templates: FunctionTemplates<'env>,
    function_instantiations: FunctionInstantiations<'env>,
    function_bodies: FunctionBodies<'env>,
    entry_point: EntryPoint<'env>,
}

impl<'env> Module<'env> {
    pub fn from_valid_module(mut module: il4il::validation::ValidModule<'env>, environment: &'env Context) -> Self {
        let _symbols = module.take_symbols();
        let contents = module.into_contents();

        Self {
            environment,
            types: Types::new(contents.types),
            function_definitions: FunctionDefinitions::new(contents.function_definitions),
            function_templates: FunctionTemplates::new(contents.function_templates),
            function_instantiations: FunctionInstantiations::new(contents.function_instantiations),
            function_bodies: FunctionBodies::new(contents.function_bodies),
            entry_point: EntryPoint::new(contents.entry_point.first().copied()),
        }
    }

    pub fn environment(&'env self) -> &'env Context {
        self.environment
    }

    pub fn types(&'env self) -> &'env [types::Type<'env>] {
        self.types
            .get_or_create(|types| types.into_iter().map(|ty| types::Type::new(self, ty)).collect())
    }

    /// Gets this module's function definitions.
    pub fn function_definitions(&'env self) -> &'env [function::template::Definition<'env>] {
        self.function_definitions.get_or_create(|definitions| {
            definitions
                .into_iter()
                .map(|definition| function::template::Definition::new(self, definition))
                .collect()
        })
    }

    /// Gets this module's function templates.
    pub fn function_templates(&'env self) -> &'env [function::template::Template<'env>] {
        self.function_templates.get_or_create(|lookup| {
            lookup
                .into_templates()
                .map(|template| function::template::Template::new(self, template))
                .collect()
        })
    }

    pub fn function_instantiations(&'env self) -> &'env [function::Instantiation<'env>] {
        self.function_instantiations.get_or_create(|instantiations| {
            instantiations
                .into_iter()
                .map(|inst| function::Instantiation::new(self, inst))
                .collect()
        })
    }

    /// Gets this module's function bodies.
    pub fn function_bodies(&'env self) -> &'env [code::Code<'env>] {
        self.function_bodies.get_or_create(|bodies| {
            bodies
                .into_iter()
                .enumerate()
                .map(|(index, body)| code::Code::new(self, index.into(), body))
                .collect()
        })
    }

    /// Gets the module's entry point function, or `None` it exists.
    pub fn entry_point(&'env self) -> Option<&'env function::Instantiation<'env>> {
        *self.entry_point.get_or_create(|index| index.map(|index| &self.function_instantiations()[usize::from(index)]))
    }
}

impl<'env> Debug for &'env Module<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("environment", self.environment)
            .field("function_definitions", &LazyDebug(&self.function_definitions))
            .field("function_instantiations", &LazyDebug(&self.function_instantiations))
            .field("function_bodies", &LazyDebug(&self.function_bodies))
            .finish_non_exhaustive()
    }
}
