//! Module to perform validation of IL4IL code.
//!
//! Validation ensures that the contents of an IL4IL module are semantically correct. Additionally, validation does not require the
//! resolution of any imports.

#![deny(unsafe_code)]

mod contents;
mod index_checker;
mod instruction_checker;
mod type_comparer;
mod type_resolver;
mod value_checker;

pub use contents::ModuleContents;

/// Error type used when a SAILAR module is not valid.
///
/// Used with [`error_stack::Report`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
#[error("module validation failed")]
pub struct ValidationError;

/// The result type used for validation.
pub type Result<T> = error_stack::Result<T, ValidationError>;

/// Represents a validated SAILAR module.
#[derive(Clone, Default)]
pub struct ValidModule<'data> {
    contents: ModuleContents<'data>,
    symbols: crate::symbol::Lookup<'data>,
}

impl<'data> ValidModule<'data> {
    /// Creates a valid module with the specified `contents`, without actually performing any validation.
    ///
    /// Using an invalid module may result in panics later.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the module is valid, though an invalid module will at worst only result in panics.
    #[allow(unsafe_code)]
    #[must_use]
    pub unsafe fn from_contents_unchecked(contents: ModuleContents<'data>) -> Self {
        Self {
            contents,
            symbols: Default::default(),
        }
    }

    pub fn contents(&self) -> &ModuleContents<'data> {
        &self.contents
    }

    pub fn symbol_lookup(&self) -> &crate::symbol::Lookup<'data> {
        &self.symbols
    }

    pub fn into_contents(self) -> ModuleContents<'data> {
        self.contents
    }

    pub fn take_symbols(&mut self) -> crate::symbol::Lookup<'data> {
        std::mem::take(&mut self.symbols)
    }

    /// Validates the given module contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the module contents are invalid.
    pub fn from_module_contents(contents: ModuleContents<'data>) -> Result<Self> {
        use error_stack::{IntoReport, ResultExt};

        //// TODO: Check that the types are valid
        //contents
        //    .function_signatures
        //    .iter()
        //    .flat_map(|signature| signature.all_types())
        //    .try_for_each(&validate_type)?;

        let symbols = crate::symbol::Lookup::from_assignments(contents.symbols.iter())
            .report()
            .change_context(ValidationError)?;

        for (index, entry) in symbols.entries().enumerate() {
            match entry.index() {
                crate::symbol::TargetIndex::FunctionTemplate(template) => index_checker::get_function_template(template, &contents),
            }
            .change_context(ValidationError)
            .attach_printable_lazy(|| format!("symbol entry #{index} ({:?}) is invalid", entry.name()))?;
        }

        // TODO: Check that template lookup is valid
        //contents.function_templates.iter

        for (index, instantiation) in contents.function_instantiations.iter().enumerate() {
            index_checker::get_function_template(instantiation.template, &contents)
                .change_context(ValidationError)
                .attach_printable_lazy(|| format!("function instantiation #{index} has an invalid template"))?;
        }

        let mut type_buffer = Vec::new();
        let mut type_buffer_2 = Vec::new();

        for (body_index, body) in contents.function_bodies.iter().enumerate() {
            instruction_checker::validate_body(body, &contents, &mut type_buffer)
                .change_context(ValidationError)
                .attach_printable_lazy(|| format!("function body #{body_index} is invalid"))?;
        }

        for (definition_index, definition) in contents.function_definitions.iter().enumerate() {
            let signature = index_checker::get_function_signatures(definition.signature, &contents)
                .change_context(ValidationError)
                .attach_printable_lazy(|| format!("function definition #{definition_index} has an invalid signature"))?;

            let body = index_checker::get_function_body(definition.body, &contents)
                .change_context(ValidationError)
                .attach_printable_lazy(|| format!("function definition #{definition_index} has an invalid body"))?;

            let expected_parameter_types = type_resolver::resolve_many(signature.parameter_types(), &mut type_buffer, &contents)
                .change_context(ValidationError)
                .attach_printable_lazy(|| format!("function definition #{definition_index} has invalid input types"))?;

            let actual_parameter_types = type_resolver::resolve_many(body.entry_block.input_types(), &mut type_buffer_2, &contents)
                .change_context(ValidationError)
                .attach_printable_lazy(|| {
                    format!("could not obtain entry block input types for function definition #{definition_index}")
                })?;

            if actual_parameter_types.len() != expected_parameter_types.len() {
                return Err(error_stack::Report::new(ValidationError)).attach_printable_lazy(|| {
                    format!(
                        "function definition {definition_index} was expected to have {} parameters, but entry block defines {} inputs",
                        expected_parameter_types.len(),
                        actual_parameter_types.len()
                    )
                });
            }

            for (index, (expected, actual)) in expected_parameter_types.iter().zip(actual_parameter_types).enumerate() {
                type_comparer::are_equal(expected, actual, &contents)
                    .report()
                    .change_context(ValidationError)
                    .attach_printable_lazy(|| format!("function parameter #{index} in definition #{definition_index} is invalid"))?;
            }

            // TODO: How to check that block result types match function signature?
        }

        if contents.entry_point.len() > 1 {
            #[derive(Debug, thiserror::Error)]
            #[error("bad entry point")]
            struct InvalidEntryPointError;

            return Err(InvalidEntryPointError)
                .report()
                .attach_printable("too many entry point functions specified")
                .change_context(ValidationError);
        }

        Ok(Self { contents, symbols })
    }
}

impl<'data> TryFrom<ModuleContents<'data>> for ValidModule<'data> {
    type Error = error_stack::Report<ValidationError>;

    fn try_from(value: ModuleContents<'data>) -> Result<Self> {
        Self::from_module_contents(value)
    }
}

impl<'data> TryFrom<crate::module::Module<'data>> for ValidModule<'data> {
    type Error = error_stack::Report<ValidationError>;

    fn try_from(value: crate::module::Module<'data>) -> Result<Self> {
        Self::from_module_contents(ModuleContents::from_module(value))
    }
}
