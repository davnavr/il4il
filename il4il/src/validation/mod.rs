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
        use crate::instruction::value::{self, Value};
        use crate::instruction::Instruction;
        use crate::type_system;

        fn maximum_index(length: usize) -> Option<usize> {
            if length == 0 {
                None
            } else {
                Some(length - 1)
            }
        }

        fn create_index_validator<S: index::IndexSpace>(length: usize) -> impl Fn(index::Index<S>) -> Result<(), Error> {
            move |index| {
                if usize::from(index) >= length {
                    return Err(Error::from_kind(InvalidIndexError::new(index, maximum_index(length))));
                }
                Ok(())
            }
        }

        let validate_type_index = create_index_validator::<index::TypeSpace>(contents.types.len());
        let validate_function_signature_index = create_index_validator(contents.function_signatures.len());

        let validate_function_template_index = create_index_validator::<index::FunctionTemplateSpace>(contents.function_templates.count());

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
        //contents.function_templates.iter

        contents
            .function_instantiations
            .iter()
            .try_for_each(|instantiation| validate_function_template_index(instantiation.template))?;

        let validate_function_body_index = create_index_validator::<index::CodeSpace>(contents.function_bodies.len());

        let validate_typed_value = |value: &Value, expected_type: type_system::Type| match value {
            Value::Constant(value::Constant::Integer(_)) => {
                if let type_system::Type::Integer(_) = expected_type {
                    Result::<_, Error>::Ok(())
                } else {
                    todo!("err for int value for non int type")
                }
            }
            Value::Constant(value::Constant::Float(float_value)) => {
                if let type_system::Type::Float(float_type) = expected_type {
                    if float_type.bit_width() == float_value.bit_width() {
                        Ok(())
                    } else {
                        todo!("error for float bit width mismatch between value and type")
                    }
                } else {
                    todo!("error for float value for non float type")
                }
            }
        };

        let validate_rtyped_value = |value: &Value, expected_type: &type_system::Reference| {
            let actual_expected_type = match expected_type {
                type_system::Reference::Index(index) => contents
                    .types
                    .get(usize::from(*index))
                    .ok_or_else(|| Error::from_kind(InvalidIndexError::new(*index, maximum_index(contents.types.len()))))?,
                type_system::Reference::Inline(ty) => ty,
            };
            (&validate_typed_value)(value, *actual_expected_type)
        };

        // TODO: Avoid code duplication with validate_type
        let resolve_type = |ty: &type_system::Reference| match ty {
            type_system::Reference::Inline(t) => Ok(*t),
            type_system::Reference::Index(index) => contents
                .types
                .get(usize::from(*index))
                .ok_or_else(|| Error::from_kind(InvalidIndexError::new(*index, maximum_index(contents.types.len()))))
                .copied(),
        };

        // TODO: Return a slice so the buffer is reused.
        let resolve_many_types = {
            let mut type_buffer = Vec::<type_system::Type>::new();
            let resolve_type = &resolve_type;
            move |types: &[type_system::Reference]| -> Result<_, Error> {
                type_buffer.clear();
                for ty in types.iter() {
                    type_buffer.push(resolve_type(ty)?);
                }
                Ok(type_buffer.into_boxed_slice())
            }
        };

        let a = &resolve_many_types;

        for body in contents.function_bodies.iter() {
            // TODO: Create a lookup to allow easy retrieval of entry block's input and result types
            for (actual_block_index, block) in body.iter_blocks().enumerate() {
                let block_index = index::Block::from(actual_block_index);
                //let expected_result_types = (a)(body.entry_block().result_types())?;

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
                        Instruction::Return(values) => todo!("validate values"),
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

        if contents.entry_point.len() > 1 {
            todo!("error for too many entry points");
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
