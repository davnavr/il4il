//! Contains types that model the contents of a valid IL4IL module.

use crate::function;
use crate::module::section::{self, Section};
use crate::module::Module;
use crate::type_system;

/// Represents the contents of a SAILAR module.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ModuleContents<'data> {
    pub metadata: Vec<section::Metadata<'data>>,
    pub types: Vec<type_system::Type>,
    pub function_signatures: Vec<function::Signature>,
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

        for sect in sections.into_iter() {
            match sect? {
                Section::Metadata(mut metadata) => contents.metadata.append(&mut metadata),
                Section::Type(mut types) => contents.types.append(&mut types),
                Section::FunctionSignature(mut signatures) => contents.function_signatures.append(&mut signatures),
                Section::Symbol(symbols) => todo!("symbol content is not yet supported"),
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
