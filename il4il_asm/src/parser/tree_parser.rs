//! Turns a tree containing nodes into an abstract syntax tree which is the final output of the parsing process.

use crate::error::Error;
use crate::syntax::{structure, tree, Located};

pub(super) fn parse<'src>(
    tree: crate::syntax::structure::Tree<'src>,
    offsets: &crate::lexer::Offsets,
    errors: &mut Vec<Error>,
) -> tree::Root<'src> {
    let mut directives = Vec::with_capacity(tree.contents.len());
    for top_node in tree.contents.into_iter() {
        match top_node.node.kind.node {
            structure::NodeKind::Directive(directive) => match directive {
                "section" => {
                    let mut attributes;
                    let contents;

                    match top_node.node.contents {
                        structure::NodeContents::Line(attrs) => {
                            attributes = attrs.into_iter();
                            contents = Vec::default().into_iter();
                        }
                        structure::NodeContents::Block { attributes: attrs, nodes } => {
                            attributes = attrs.into_iter();
                            contents = nodes.into_iter();
                        }
                    }

                    match attributes.next() {
                        Some(Located {
                            node: structure::Attribute::Word(kind),
                            offsets: location,
                        }) => match kind {
                            "metadata" => {
                                todo!("parse metadata")
                            }
                            _ => errors.push(Error::from_string(
                                offsets.get_location_range(location),
                                format!("\"{kind}\" is not a valid section kind"),
                            )),
                        },
                        kind => {
                            errors.push(Error::from_str(
                                offsets.get_location_range(kind.map(|n| n.offsets).unwrap_or(top_node.node.kind.offsets)),
                                "expected section kind",
                            ));
                        }
                    };

                    for Located { offsets: location, .. } in attributes {
                        errors.push(Error::from_str(
                            offsets.get_location_range(location),
                            "unexpected section attribute",
                        ));
                    }
                }
                _ => errors.push(Error::from_string(
                    offsets.get_location_range(top_node.node.kind.offsets),
                    format!("unknown directive \"{directive}\", expected \".section\""),
                )),
            },
            structure::NodeKind::Word(word) => errors.push(Error::from_string(
                offsets.get_location_range(top_node.offsets),
                format!("unexpected word {word}, expected directive"),
            )),
        }
    }

    tree::Root { directives }
}
