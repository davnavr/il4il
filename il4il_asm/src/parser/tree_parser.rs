//! Turns a tree containing nodes into an abstract syntax tree which is the final output of the parsing process.

use crate::syntax::{structure, tree, Located};

pub(super) fn parse<'src>(tree: crate::syntax::structure::Tree<'src>, context: &mut crate::parser::Context<'_>) -> tree::Root<'src> {
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
                                let mut metadata = Vec::new();
                                for content_node in contents {
                                    match content_node.node.kind.node {
                                        structure::NodeKind::Directive("name") => {
                                            todo!()
                                        }
                                        _ => todo!(),
                                    }
                                }

                                directives.push(Located::new(
                                    tree::TopLevelDirective::Section(tree::SectionDefinition::Metadata(metadata)),
                                    top_node.offsets,
                                ))
                            }
                            _ => {
                                let kind = kind.to_string();
                                context.push_error_at(location, move |f| write!(f, "\"{kind}\" is not a valid section kind"))
                            }
                        },
                        kind => {
                            context.push_error_str_at(
                                kind.map(|n| n.offsets).unwrap_or(top_node.node.kind.offsets),
                                "expected section kind",
                            );
                        }
                    };

                    for Located { offsets: location, .. } in attributes {
                        context.push_error_str_at(location, "unexpected section attribute");
                    }
                }
                _ => context.push_error_string_at(
                    top_node.node.kind.offsets,
                    format!("unknown directive \"{directive}\", expected \".section\""),
                ),
            },
            structure::NodeKind::Word(word) => {
                context.push_error_string_at(top_node.offsets, format!("unexpected word {word}, expected directive"))
            }
        }
    }

    tree::Root { directives }
}
