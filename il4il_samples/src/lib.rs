//! Contains functions to generate sample IL4IL modules.

use il4il::function;
use il4il::identifier::Id;
use il4il::index::Index;
use il4il::instruction::{self, Instruction};
use il4il::module::section::{Metadata, Section};
use il4il::module::{Module, ModuleName};
use il4il::type_system;
use il4il::validation;

/// Creates an IL4IL module containing an entry point function that simply returns an integer exit code.
///
/// # Examples
///
/// ```
/// il4il_samples::return_int("ok", 0);
/// ```
pub fn return_int(name: &'static str, exit_code: i32) -> validation::ValidModule<'static> {
    let mut builder = Module::new();
    let s32_type = type_system::Reference::from(type_system::SizedInteger::S32);
    *builder.sections_mut() = vec![
        Section::Metadata(vec![Metadata::Name(ModuleName::from_name(Id::new(name).unwrap()))]),
        Section::Code(vec![function::Body::new(
            vec![s32_type.clone()].into_boxed_slice(),
            instruction::Block::new([], [], vec![Instruction::Return(Box::new([exit_code.into()]) as Box<[_]>)]),
            Default::default(),
        )]),
        Section::FunctionSignature(vec![function::Signature::new([s32_type], [])]),
        Section::FunctionDefinition(vec![function::Definition::new(Index::new(0), Index::new(0))]),
        Section::FunctionInstantiation(vec![function::Instantiation::with_template(Index::new(0))]),
        Section::EntryPoint(Index::new(0)),
    ];
    validation::ValidModule::try_from(builder).unwrap()
}
