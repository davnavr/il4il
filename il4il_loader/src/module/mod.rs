//! Contains the [`Module`] struct.

mod import;

pub use import::Import;

use crate::code;
use crate::debug::LazyDebug;
use crate::environment::Context;
use crate::function;
use crate::types;
use std::fmt::{Debug, Formatter};

type Types<'env> = lazy_init::LazyTransform<Vec<il4il::type_system::Type>, Box<[types::Type<'env>]>>;

type FunctionSignatures<'env> = lazy_init::LazyTransform<Vec<il4il::function::Signature>, Box<[function::Signature<'env>]>>;

type FunctionDefinitions<'env> = lazy_init::LazyTransform<Vec<il4il::function::Definition>, Box<[function::template::Definition<'env>]>>;

type FunctionImports<'env> = lazy_init::LazyTransform<Vec<il4il::function::Import<'env>>, Box<[function::template::Import<'env>]>>;

type FunctionTemplates<'env> = lazy_init::LazyTransform<il4il::function::TemplateLookup, Box<[function::template::Template<'env>]>>;

type FunctionInstantiations<'env> = lazy_init::LazyTransform<Vec<il4il::function::Instantiation>, Box<[function::Instantiation<'env>]>>;

type FunctionBodies<'env> = lazy_init::LazyTransform<Vec<il4il::function::Body>, Box<[code::Code<'env>]>>;

type ModuleImports<'env> = lazy_init::LazyTransform<Vec<il4il::module::ModuleName<'env>>, Box<[Import<'env>]>>;

type EntryPoint<'env> = lazy_init::LazyTransform<Option<il4il::index::FunctionInstantiation>, Option<&'env function::Instantiation<'env>>>;

/// Encapsulates an IL4IL module and its associated state, allowing for easy resolution of imports, types, etc.
pub struct Module<'env> {
    environment: &'env Context,
    types: Types<'env>,
    function_signatures: FunctionSignatures<'env>,
    function_definitions: FunctionDefinitions<'env>,
    function_imports: FunctionImports<'env>,
    function_templates: FunctionTemplates<'env>,
    function_instantiations: FunctionInstantiations<'env>,
    function_bodies: FunctionBodies<'env>,
    module_imports: ModuleImports<'env>,
    entry_point: EntryPoint<'env>,
}

impl<'env> Module<'env> {
    pub fn from_valid_module(mut module: il4il::validation::ValidModule<'env>, environment: &'env Context) -> Self {
        let _symbols = module.take_symbols();
        let contents = module.into_contents();

        Self {
            environment,
            types: Types::new(contents.types),
            function_signatures: FunctionSignatures::new(contents.function_signatures),
            function_definitions: FunctionDefinitions::new(contents.function_definitions),
            function_imports: FunctionImports::new(contents.function_imports),
            function_templates: FunctionTemplates::new(contents.function_templates),
            function_instantiations: FunctionInstantiations::new(contents.function_instantiations),
            function_bodies: FunctionBodies::new(contents.function_bodies),
            module_imports: ModuleImports::new(contents.module_imports),
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

    pub fn function_signatures(&'env self) -> &'env [function::Signature<'env>] {
        self.function_signatures
            .get_or_create(|signatures| signatures.into_iter().map(|sig| function::Signature::new(self, sig)).collect())
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

    pub fn function_imports(&'env self) -> &'env [function::template::Import<'env>] {
        self.function_imports.get_or_create(|imports| {
            imports
                .into_iter()
                .map(|func| function::template::Import::new(self, func))
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

    pub fn module_imports(&'env self) -> &'env [Import<'env>] {
        self.module_imports
            .get_or_create(|imports| imports.into_iter().map(|name| Import::new(self, name)).collect())
    }

    /// Gets the module's entry point function, or `None` it exists.
    pub fn entry_point(&'env self) -> Option<&'env function::Instantiation<'env>> {
        *self
            .entry_point
            .get_or_create(|index| index.map(|index| &self.function_instantiations()[usize::from(index)]))
    }
}

impl Debug for Module<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("environment", self.environment)
            .field("function_definitions", &LazyDebug(&self.function_definitions))
            .field("function_instantiations", &LazyDebug(&self.function_instantiations))
            .field("function_bodies", &LazyDebug(&self.function_bodies))
            .finish_non_exhaustive()
    }
}
