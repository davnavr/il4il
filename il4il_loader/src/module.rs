//! Contains the [`Module`] struct.

use crate::code;
use crate::debug::LazyDebug;
use crate::environment::Context;
use crate::function;
use std::fmt::{Debug, Formatter};

type FunctionDefinitions<'env> = lazy_init::LazyTransform<Vec<il4il::function::Definition>, Box<[function::template::Definition<'env>]>>;

type FunctionBodies<'env> = lazy_init::LazyTransform<Vec<il4il::function::Body>, Box<[code::Code<'env>]>>;

pub struct Module<'env> {
    environment: &'env Context,
    function_definitions: FunctionDefinitions<'env>,
    function_bodies: FunctionBodies<'env>,
}

impl<'env> Module<'env> {
    pub fn from_valid_module(mut module: il4il::validation::ValidModule<'env>, environment: &'env Context) -> Self {
        let _symbols = module.take_symbols();
        let contents = module.into_contents();

        Self {
            environment,
            function_definitions: FunctionDefinitions::new(contents.function_definitions),
            function_bodies: FunctionBodies::new(contents.function_bodies),
        }
    }

    pub fn environment(&'env self) -> &'env Context {
        self.environment
    }

    pub fn function_definitions(&'env self) -> &'env [function::template::Definition<'env>] {
        self.function_definitions.get_or_create(|definitions| {
            definitions
                .into_iter()
                .map(|definition| function::template::Definition::new(self, definition))
                .collect()
        })
    }

    pub fn function_bodies(&'env self) -> &'env [code::Code<'env>] {
        self.function_bodies.get_or_create(|bodies| {
            bodies
                .into_iter()
                .enumerate()
                .map(|(index, body)| code::Code::new(self, index.into(), body))
                .collect()
        })
    }
}

impl<'env> Debug for &'env Module<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("environment", self.environment)
            .field("function_definitions", &LazyDebug(&self.function_definitions))
            .field("function_bodies", &LazyDebug(&self.function_bodies))
            .finish_non_exhaustive()
    }
}
