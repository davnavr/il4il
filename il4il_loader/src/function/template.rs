//! Contains types representing IL4IL function templates.

use crate::code::Code;
use crate::module::Module;
use std::fmt::{Debug, Formatter};

type Body<'env> = lazy_init::LazyTransform<il4il::index::FunctionBody, &'env Code<'env>>;

#[repr(transparent)]
struct BodyDebug<'a, 'env>(&'a Body<'env>);

impl Debug for BodyDebug<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0.get().map(|body| body.index()), f)
    }
}

/// A function definition defined in the current module.
pub struct Definition<'env> {
    module: &'env Module<'env>,
    body: Body<'env>,
}

impl<'env> Definition<'env> {
    pub(crate) fn new(module: &'env Module<'env>, definition: il4il::function::Definition) -> Self {
        Self {
            module,
            body: Body::new(definition.body),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn body(&'env self) -> &'env Code<'env> {
        self.body.get_or_create(|index| &self.module.function_bodies()[usize::from(index)])
    }
}

impl Debug for Definition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Definition")
            .field("body", &BodyDebug(&self.body))
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum TemplateKind<'env> {
    Definition(&'env Definition<'env>),
}

pub struct Template<'env> {
    module: &'env Module<'env>,
    kind: lazy_init::LazyTransform<il4il::function::Template, TemplateKind<'env>>,
}

impl<'env> Template<'env> {
    pub(crate) fn new(module: &'env Module<'env>, template: il4il::function::Template) -> Self {
        Self {
            module,
            kind: lazy_init::LazyTransform::new(template),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn kind(&'env self) -> &'env TemplateKind<'env> {
        self.kind.get_or_create(|template| match template {
            il4il::function::Template::Definition(index) => TemplateKind::Definition(&self.module.function_definitions()[index]),
            il4il::function::Template::Import(index) => todo!("handle function imports"),
        })
    }
}

impl Debug for Template<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("kind", &crate::debug::LazyDebug(&self.kind))
            .finish_non_exhaustive()
    }
}
