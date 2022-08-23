//! Module to perform validation of IL4IL code.
//!
//! Validation ensures that the contents of an IL4IL module are semantically correct. Additionally, validation does not require the
//! resolution of any imports.

#![deny(unsafe_code)]

mod contents;
mod error;

pub use contents::ModuleContents;
pub use error::*;

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
    pub fn from_module_contents(contents: ModuleContents<'data>) -> Result<Self, Error> {
        use crate::index;
        use crate::instruction::Instruction;
        use crate::type_system;

        fn create_index_validator<S: index::IndexSpace>(length: usize) -> impl Fn(index::Index<S>) -> Result<(), Error> {
            let maximum = if length == 0 { None } else { Some(length - 1) };
            move |index| {
                if usize::from(index) >= length {
                    return Err(Error::from_kind(InvalidIndexError::new(index, maximum)));
                }
                Ok(())
            }
        }

        let validate_type_index = create_index_validator::<index::TypeSpace>(contents.types.len());
        let validate_function_signature_index = create_index_validator(contents.function_signatures.len());

        let validate_function_template_index = |_: index::FunctionTemplate| -> Result<(), Error> {
            todo!("add function template lookup thing to contents (function_templates: HashMap<index::FunctionTemplate, SomeEnumIndex>)")
        };

        let validate_type = |ty: &type_system::Reference| {
            match ty {
                type_system::Reference::Index(index) => (validate_type_index)(*index),
                type_system::Reference::Inline(_) => Ok(()), // TODO: When types can have type indices, validate here as well.
            }
        };

        contents
            .function_signatures
            .iter()
            .flat_map(|signature| signature.all_types())
            .try_for_each(&validate_type)?;

        let symbols = crate::symbol::Lookup::from_assignments(contents.symbols.iter())?;

        symbols.entries().try_for_each(|entry| match entry.index() {
            crate::symbol::TargetIndex::FunctionTemplate(template) => (validate_function_template_index)(template),
        })?;

        // TODO: Check that template lookup is valid

        let validate_function_body_index = create_index_validator::<index::CodeSpace>(contents.function_bodies.len());
        for body in contents.function_bodies.iter() {
            // TODO: Create a lookup to allow easy retrieval of entry block's input and result types
            for (actual_block_index, block) in body.iter_blocks().enumerate() {
                let block_index = index::Block::from(actual_block_index);

                let instruction_location = std::cell::RefCell::new(Option::<(usize, &Instruction)>::None);

                let invalid_instruction = |kind: InvalidInstructionKind| {
                    Error::from_kind(InvalidInstructionError::new(
                        block_index,
                        instruction_location
                            .take()
                            .map(|(index, instruction)| InvalidInstructionLocation::new(instruction.clone(), index)),
                        kind,
                    ))
                };

                let mut reached_terminator = false;

                for location @ (_, instruction) in block.instructions.iter().enumerate() {
                    instruction_location.replace(Some(location));

                    if reached_terminator {
                        return Err(invalid_instruction(InvalidInstructionKind::ExpectedTerminatorAsLastInstruction));
                    }

                    match instruction {
                        Instruction::Unreachable => (),
                        Instruction::Return(_values) => todo!("validate values"),
                    }

                    reached_terminator = instruction.is_terminator();
                }

                if !reached_terminator {
                    return Err(invalid_instruction(InvalidInstructionKind::ExpectedTerminatorAsLastInstruction));
                }
            }
        }

        for definition in contents.function_definitions.iter() {
            validate_function_signature_index(definition.signature)?;
            validate_function_body_index(definition.body)?;
            // TODO: How to check that entry block inputs and results match function signature?
        }

        Ok(Self { contents, symbols })
    }
}

impl<'data> TryFrom<ModuleContents<'data>> for ValidModule<'data> {
    type Error = Error;

    fn try_from(value: ModuleContents<'data>) -> Result<Self, Error> {
        Self::from_module_contents(value)
    }
}

impl<'data> TryFrom<crate::module::Module<'data>> for ValidModule<'data> {
    type Error = Error;

    fn try_from(value: crate::module::Module<'data>) -> Result<Self, Error> {
        Self::from_module_contents(ModuleContents::from_module(value))
    }
}
