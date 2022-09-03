//! Provides functions to validate the contents of function bodies.

use crate::function;
use crate::index;
use crate::instruction::{self, Instruction};
use crate::type_system;
use crate::validation::type_resolver;
use crate::validation::value_checker;
use error_stack::ResultExt;

/// Indicates the location of an invalid instruction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidInstructionLocation {
    instruction: Instruction,
    index: usize,
}

impl InvalidInstructionLocation {
    fn new(instruction: Instruction, index: usize) -> Self {
        Self { instruction, index }
    }
}

/// Error type used when an invalid instruction is encountered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidInstructionError {
    block_index: index::Block,
    location: Option<InvalidInstructionLocation>,
}

impl InvalidInstructionError {
    fn new(block: index::Block, location: Option<InvalidInstructionLocation>) -> Self {
        Self {
            block_index: block,
            location,
        }
    }
}

impl std::fmt::Display for InvalidInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.location {
            write!(
                f,
                "invalid instruction {:?} at index {} in block {}",
                location.instruction, location.index, self.block_index
            )
        } else {
            write!(f, "block {} is invalid", self.block_index)
        }
    }
}

impl std::error::Error for InvalidInstructionError {}

pub(crate) fn validate_body(
    body: &function::Body,
    contents: &crate::validation::ModuleContents,
    type_buffer: &mut Vec<type_system::Type>,
) -> error_stack::Result<(), InvalidInstructionError> {
    for (actual_block_index, block) in body.iter_blocks().enumerate() {
        let block_index = crate::index::Block::from(actual_block_index);
        let current_location = std::cell::RefCell::<Option<(usize, &Instruction)>>::new(None);

        let encountered_invalid = || {
            InvalidInstructionError::new(
                block_index,
                current_location
                    .take()
                    .map(|(index, instruction)| InvalidInstructionLocation::new(instruction.clone(), index)),
            )
        };

        let report_invalid = || error_stack::Report::new(encountered_invalid());

        // TODO: Result types should be defined in the body, shared across all blocks.
        let expected_result_types = type_resolver::resolve_many(body.entry_block().result_types(), type_buffer, contents)
            .change_context_lazy(encountered_invalid)
            .attach_printable("result types are invalid")?;

        let mut reached_terminator = false;
        for location @ (_, instruction) in block.instructions.iter().enumerate() {
            current_location.replace(Some(location));

            if reached_terminator {
                return Err(report_invalid().attach_printable("cannot have instructions after the first terminator instruction"));
            }

            match instruction {
                Instruction::Unreachable => (),
                Instruction::Return(values) => {
                    if values.len() != expected_result_types.len() {
                        return Err(report_invalid()).attach_printable_lazy(|| {
                            format!("expected {} return values, but got {}", expected_result_types.len(), values.len())
                        });
                    }

                    value_checker::check_values_iter(values.iter().zip(expected_result_types.iter()), contents)
                        .change_context_lazy(encountered_invalid)
                        .attach_printable("return values are invalid")?;
                }
            }

            reached_terminator = instruction.is_terminator();
        }

        if !reached_terminator {
            return Err(report_invalid().attach_printable("expected terminator instruction at end of block"));
        }
    }

    Ok(())
}
