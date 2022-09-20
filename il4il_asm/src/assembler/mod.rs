//! The IL4IL assembler takes a syntax tree and produces an in-memory representation of an IL4IL module.

use crate::error::Error;
use crate::syntax::tree;
use il4il::module;

pub type Output<'cache> = module::Module<'cache>;

pub fn assemble<'cache>(inputs: crate::parser::Output<'cache>, errors: &mut Vec<Error>) -> Output<'cache> {
    let mut module = module::Module::new();
    let mut sections = Vec::with_capacity(inputs.tree.directives.len());

    for top_directive in inputs.tree.directives.into_iter() {
        match top_directive.node {
            tree::TopLevelDirective::Section(section) => sections.push(match section {
                tree::SectionDefinition::Metadata(metadata) => {
                    let mut entries = Vec::with_capacity(metadata.len());
                    for m in metadata.into_iter() {
                        match m.node {
                            tree::MetadataDirective::Name(name) => entries.push(module::section::Metadata::Name(name.node)),
                        }
                    }
                    module::section::Section::Metadata(entries)
                }
            }),
        }
    }

    *module.sections_mut() = sections;
    module
}
