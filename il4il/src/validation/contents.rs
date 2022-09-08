//! Contains types that model the contents of a valid IL4IL module.

use crate::function;
use crate::module::section::{self, Section};
use crate::module::{Module, ModuleName};
use crate::type_system;

/// Represents the contents of a SAILAR module.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ModuleContents<'data> {
    pub metadata: Vec<section::Metadata<'data>>,
    pub symbols: Vec<crate::symbol::Assignment<'data>>,
    pub types: Vec<type_system::Type>,
    pub function_signatures: Vec<function::Signature>,
    pub function_imports: Vec<function::Import<'data>>,
    pub function_definitions: Vec<function::Definition>,
    pub function_bodies: Vec<function::Body>,
    pub function_templates: function::TemplateLookup,
    pub function_instantiations: Vec<function::Instantiation>,
    pub entry_point: Vec<crate::index::FunctionInstantiation>,
    pub module_imports: Vec<ModuleName<'data>>,
}

impl<'data> ModuleContents<'data> {
    /// Creates an empty SAILAR module.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_sections_fallible<S, E>(sections: S) -> Result<Self, E>
    where
        S: IntoIterator<Item = Result<Section<'data>, E>>,
    {
        let mut contents = ModuleContents::default();
        let mut function_import_index = 0;
        let mut function_definition_index = 0;

        for sect in sections.into_iter() {
            match sect? {
                Section::Metadata(mut metadata) => contents.metadata.append(&mut metadata),
                Section::Symbol(mut symbols) => contents.symbols.append(&mut symbols),
                Section::Type(mut types) => contents.types.append(&mut types),
                Section::FunctionSignature(mut signatures) => contents.function_signatures.append(&mut signatures),
                Section::FunctionInstantiation(mut instantiations) => contents.function_instantiations.append(&mut instantiations),
                Section::FunctionImport(imports) => {
                    contents.function_templates.reserve(imports.len());
                    contents.function_imports.reserve(imports.len());
                    for func in imports {
                        contents
                            .function_templates
                            .insert(function::Template::Import(function_import_index));
                        function_import_index += 1;
                        contents.function_imports.push(func);
                    }
                }
                Section::FunctionDefinition(definitions) => {
                    contents.function_templates.reserve(definitions.len());
                    contents.function_definitions.reserve(definitions.len());
                    for func in definitions {
                        contents
                            .function_templates
                            .insert(function::Template::Definition(function_definition_index));
                        function_definition_index += 1;
                        contents.function_definitions.push(func);
                    }
                }
                Section::Code(mut code) => contents.function_bodies.append(&mut code),
                Section::EntryPoint(index) => contents.entry_point.push(index),
                Section::ModuleImport(mut modules) => contents.module_imports.append(&mut modules),
            }
        }

        Ok(contents)
    }

    #[must_use]
    pub fn from_sections<S: IntoIterator<Item = Section<'data>>>(sections: S) -> Self {
        Self::from_sections_fallible(sections.into_iter().map(Result::<_, std::convert::Infallible>::Ok)).unwrap()
    }

    #[must_use]
    pub fn from_module(module: Module<'data>) -> Self {
        Self::from_sections(module.into_sections())
    }

    /// Converts the module contents into a sequence of sections.
    #[must_use]
    pub fn into_sections(self) -> Box<[Section<'data>]> {
        let mut sections = {
            let mut capacity = 0;

            if !self.metadata.is_empty() {
                capacity += 1;
            }

            if !self.types.is_empty() {
                capacity += 1;
            }

            Vec::with_capacity(capacity)
        };

        // TODO: For some sections, may need to rearrange order, so this might not work correctly.

        sections.push(Section::Metadata(self.metadata));
        sections.push(Section::Type(self.types));
        sections.into_boxed_slice()
    }

    #[must_use]
    pub fn into_module(self) -> Module<'data> {
        let mut module = Module::new();
        *module.sections_mut() = self.into_sections().into_vec();
        module
    }
}

impl<'data> From<Module<'data>> for ModuleContents<'data> {
    fn from(module: Module<'data>) -> Self {
        Self::from_module(module)
    }
}

impl<'data> From<ModuleContents<'data>> for Module<'data> {
    fn from(contents: ModuleContents<'data>) -> Self {
        contents.into_module()
    }
}
