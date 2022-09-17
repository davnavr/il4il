//! Turns a tree containing nodes into an abstract syntax tree which is the final output of the parsing process.

use crate::error::{Error, Result};
use crate::lexer::Offsets;
use crate::parser::Context;
use crate::syntax::{structure, tree, Located};
use std::fmt::Formatter;
use std::ops::Range;

fn error_unexpected<N: ToString>(node: Located<N>, context: &mut Context<'_>) {
    let Located { node: content, offsets } = node;
    let s = content.to_string();
    context.push_error_at(offsets, move |f: &mut Formatter| write!(f, "unexpected \"{s}\""))
}

struct AttributeParser<'src> {
    attributes: std::vec::IntoIter<Located<structure::Attribute<'src>>>,
}

impl<'src> AttributeParser<'src> {
    fn expect_any(
        &mut self,
        offsets: &Offsets,
        default_offset: &Range<usize>,
        error: &'static str,
    ) -> Result<Located<structure::Attribute<'src>>> {
        if let Some(attribute) = self.attributes.next() {
            Ok(attribute)
        } else {
            Err(Error::new(
                offsets.get_location_range(default_offset.clone()),
                move |f: &mut Formatter| write!(f, "{error}, unexpected end"),
            ))
        }
    }

    fn expect_word(&mut self, offsets: &Offsets, default_offset: &Range<usize>, error: &'static str) -> Result<Located<&'src str>> {
        let node = self.expect_any(offsets, default_offset, &error)?;
        match node.node {
            structure::Attribute::Word(word) => Ok(Located::new(word, node.offsets)),
            bad => {
                let s = bad.to_string();
                Err(Error::new(offsets.get_location_range(node.offsets), move |f: &mut Formatter| {
                    write!(f, "{error}, but got \"{s}\"")
                }))
            }
        }
    }

    fn expect_end(self, context: &mut Context<'_>) {
        self.attributes.for_each(|bad| error_unexpected(bad, context))
    }
}

enum ContentKind {
    Empty,
    Block,
}

struct ContentParser<'src> {
    kind: ContentKind,
    contents: std::vec::IntoIter<Located<structure::Node<'src>>>,
}

fn parse_node_contents(node: structure::NodeContents) -> (AttributeParser, ContentParser) {
    let attributes;
    let content_kind;
    let contents;

    match node {
        structure::NodeContents::Line(attrs) => {
            attributes = attrs;
            content_kind = ContentKind::Empty;
            contents = Vec::new();
        }
        structure::NodeContents::Block { attributes: attrs, nodes } => {
            attributes = attrs;
            content_kind = ContentKind::Block;
            contents = nodes;
        }
    }

    (
        AttributeParser {
            attributes: attributes.into_iter(),
        },
        ContentParser {
            kind: content_kind,
            contents: contents.into_iter(),
        },
    )
}

fn parse_section<'src>(
    location: &Range<usize>,
    mut attributes: AttributeParser<'src>,
    mut contents: ContentParser<'src>,
    context: &mut Context<'_>,
) -> Result<tree::SectionDefinition<'src>> {
    let kind: Located<&'src str> = attributes.expect_word(context.offsets(), location, "expected section kind")?;
    match kind {
        _ => {
            let s = kind.node.to_string();
            Err(Error::new(
                context.offsets().get_location_range(kind.offsets),
                move |f: &mut Formatter| write!(f, "\"{s}\" is not a known section kind"),
            ))
        }
    }
}

pub(super) fn parse<'src>(tree: structure::Tree<'src>, context: &mut Context<'_>) -> tree::Root<'src> {
    let mut directives = Vec::with_capacity(tree.contents.len());
    for top_node in tree.contents.into_iter() {
        match top_node.node.kind.node {
            structure::NodeKind::Directive(directive) => {
                let (attributes, contents) = parse_node_contents(top_node.node.contents);
                match directive {
                    "section" => {
                        let r = parse_section(&top_node.node.kind.offsets, attributes, contents, context);
                        if let Some(section) = context.report_error(r) {
                            directives.push(Located::new(tree::TopLevelDirective::Section(section), top_node.offsets))
                        }
                        // let mut attributes;
                        // let contents;

                        // match top_node.node.contents {
                        //     structure::NodeContents::Line(attrs) => {
                        //         attributes = attrs.into_iter();
                        //         contents = Vec::default().into_iter();
                        //     }
                        //     structure::NodeContents::Block { attributes: attrs, nodes } => {
                        //         attributes = attrs.into_iter();
                        //         contents = nodes.into_iter();
                        //     }
                        // }

                        // match attributes.next() {
                        //     Some(Located {
                        //         node: structure::Attribute::Word(kind),
                        //         offsets: location,
                        //     }) => match kind {
                        //         "metadata" => {
                        //             let mut metadata = Vec::new();
                        //             for content_node in contents {
                        //                 match content_node.node.kind.node {
                        //                     structure::NodeKind::Directive("name") => {
                        //                         todo!()
                        //                     }
                        //                     _ => todo!(),
                        //                 }
                        //             }

                        //             directives.push(Located::new(
                        //                 tree::TopLevelDirective::Section(tree::SectionDefinition::Metadata(metadata)),
                        //                 top_node.offsets,
                        //             ))
                        //         }
                        //         _ => {
                        //             let kind = kind.to_string();
                        //             context.push_error_at(location, move |f| write!(f, "\"{kind}\" is not a valid section kind"))
                        //         }
                        //     },
                        //     kind => {
                        //         context.push_error_str_at(
                        //             kind.map(|n| n.offsets).unwrap_or(top_node.node.kind.offsets),
                        //             "expected section kind",
                        //         );
                        //     }
                        // };

                        // for Located { offsets: location, .. } in attributes {
                        //     context.push_error_str_at(location, "unexpected section attribute");
                        // }
                    }
                    _ => {
                        let directive = directive.to_string();
                        context.push_error_at(top_node.node.kind.offsets, move |f: &mut Formatter| {
                            write!(f, "unknown directive \"{directive}\", expected \".section\"")
                        })
                    }
                }
            }
            structure::NodeKind::Word(word) => {
                let word = word.to_string();
                context.push_error_at(top_node.offsets, move |f: &mut Formatter| {
                    write!(f, "unexpected word {word}, expected directive")
                })
            }
        }
    }

    tree::Root { directives }
}
