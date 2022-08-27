//! Contains functions to generate sample IL4IL modules.

use il4il::function;
use il4il::identifier::Id;
use il4il::index::Index;
use il4il::instruction::{self, Instruction};
use il4il::module::section::{Metadata, Section};
use il4il::module::Module;
use il4il::type_system;
use il4il::validation;

pub fn return_int(name: &'static str, exit_code: i32) -> validation::ValidModule<'static> {
    let mut builder = Module::new();
    let s32_type = type_system::Reference::from(type_system::SizedInteger::S32);
    *builder.sections_mut() = vec![
        Section::Metadata(vec![Metadata::Name(Id::new(name).unwrap().into())]),
        Section::Code(vec![function::Body::new(
            instruction::Block::new(
                [],
                [s32_type.clone()],
                [],
                vec![Instruction::Return(Box::new([exit_code.into()]) as Box<[_]>)],
            ),
            Default::default(),
        )]),
        Section::FunctionSignature(vec![function::Signature::new([s32_type], [])]),
        Section::FunctionDefinition(vec![function::Definition::new(Index::new(0), Index::new(0))]),
    ];
    validation::ValidModule::try_from(builder).unwrap()
}
