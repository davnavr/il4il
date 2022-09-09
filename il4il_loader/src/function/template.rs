//! Contains types representing IL4IL function templates.

use crate::code::Code;
use crate::function::signature;
use crate::module::{self, Module};
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
    signature: signature::Reference<'env>,
    body: Body<'env>,
}

impl<'env> Definition<'env> {
    pub(crate) fn new(module: &'env Module<'env>, definition: il4il::function::Definition) -> Self {
        Self {
            module,
            signature: signature::Reference::new(module, definition.signature),
            body: Body::new(definition.body),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn body(&'env self) -> &'env Code<'env> {
        self.body.get_or_create(|index| &self.module.function_bodies()[usize::from(index)])
    }

    pub fn signature(&'env self) -> &'env signature::Signature<'env> {
        self.signature.signature()
    }
}

impl Debug for Definition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Definition")
            .field("body", &BodyDebug(&self.body))
            .finish_non_exhaustive()
    }
}

pub struct Import<'env> {
    module: &'env module::Import<'env>,
    symbol: std::borrow::Cow<'env, il4il::identifier::Id>,
    signature: signature::Reference<'env>,
}

impl<'env> Import<'env> {
    pub(crate) fn new(importer: &'env Module<'env>, template: il4il::function::Import<'env>) -> Self {
        let module = &importer.module_imports()[usize::from(template.module)];
        Self {
            module,
            symbol: template.symbol,
            signature: signature::Reference::new(module.importer(), template.signature),
        }
    }

    pub fn module(&'env self) -> &'env module::Import<'env> {
        self.module
    }

    pub fn symbol(&'env self) -> &'env il4il::identifier::Id {
        self.symbol.as_ref()
    }

    pub fn signature(&'env self) -> &'env signature::Signature<'env> {
        self.signature.signature()
    }
}

impl Debug for Import<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Import").field("symbol", &self.symbol).finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum TemplateKind<'env> {
    Definition(&'env Definition<'env>),
    Import(&'env Import<'env>),
}

impl<'env> TemplateKind<'env> {
    pub fn signature(&'env self) -> &'env signature::Signature<'env> {
        match self {
            Self::Definition(definition) => definition.signature(),
            Self::Import(import) => import.signature(),
        }
    }
}

pub struct Template<'env> {
    module: &'env Module<'env>,
    index: il4il::index::FunctionTemplate,
    kind: lazy_init::LazyTransform<il4il::function::Template, TemplateKind<'env>>,
}

impl<'env> Template<'env> {
    pub(crate) fn new(module: &'env Module<'env>, index: il4il::index::FunctionTemplate, template: il4il::function::Template) -> Self {
        Self {
            module,
            index,
            kind: lazy_init::LazyTransform::new(template),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn index(&'env self) -> il4il::index::FunctionTemplate {
        self.index
    }

    pub fn kind(&'env self) -> &'env TemplateKind<'env> {
        self.kind.get_or_create(|template| match template {
            il4il::function::Template::Definition(index) => TemplateKind::Definition(&self.module.function_definitions()[index]),
            il4il::function::Template::Import(index) => TemplateKind::Import(&self.module.function_imports()[index]),
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
