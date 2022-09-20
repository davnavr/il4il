//! Turns a tree containing nodes into an abstract syntax tree which is the final output of the parsing process.

use crate::cache::StringRef;
use crate::error::{Error, Result};
use crate::lexer::Offsets;
use crate::parser::Context;
use crate::syntax::{literal, structure, tree, Located};
use std::fmt::Formatter;
use std::ops::{Deref, Range};

fn error_unexpected<N: ToString>(node: Located<N>, context: &mut Context<'_>) {
    let Located { node: content, offsets } = node;
    let s = content.to_string();
    context.push_error_at(offsets, move |f: &mut Formatter| write!(f, "unexpected \"{s}\""))
}

struct AttributeParser<S: Deref<Target = str>> {
    attributes: std::vec::IntoIter<Located<structure::Attribute<S>>>,
}

impl<'str, S: StringRef<'str>> AttributeParser<S> {
    fn expect_any(
        &mut self,
        offsets: &Offsets,
        default_offset: &Range<usize>,
        error: &'static str,
    ) -> Result<Located<structure::Attribute<S>>> {
        if let Some(attribute) = self.attributes.next() {
            Ok(attribute)
        } else {
            Err(Error::new(
                offsets.get_location_range(default_offset.clone()),
                move |f: &mut Formatter| write!(f, "{error}, unexpected end"),
            ))
        }
    }

    fn expect_word(&mut self, offsets: &Offsets, default_offset: &Range<usize>, error: &'static str) -> Result<Located<S>> {
        let node = self.expect_any(offsets, default_offset, error)?;
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

    fn expect_literal_string(&mut self, offsets: &Offsets, default_offset: &Range<usize>) -> Result<Located<literal::String<S>>> {
        let node = self.expect_any(offsets, default_offset, "expected literal string")?;
        match node.node {
            structure::Attribute::String(s) => Ok(Located::new(s, node.offsets)),
            bad => {
                let s = bad.to_string();
                Err(Error::new(offsets.get_location_range(node.offsets), move |f: &mut Formatter| {
                    write!(f, "expected literal string, but got \"{s}\"")
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
    Block, // (std::vec::IntoIter),
}

struct ContentParser<S: Deref<Target = str>> {
    kind: ContentKind,
    contents: std::vec::IntoIter<Located<structure::Node<S>>>,
}

impl<'str, S: StringRef<'str>> ContentParser<S> {
    fn expect_empty(self, context: &mut Context<'_>) {
        self.contents.for_each(|bad| error_unexpected(bad.node.kind, context))
    }
}

fn parse_node_contents<'str, S: StringRef<'str>>(node: structure::NodeContents<S>) -> (AttributeParser<S>, ContentParser<S>) {
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

fn parse_section<'str, S: StringRef<'str>>(
    location: &Range<usize>,
    mut attributes: AttributeParser<S>,
    contents: ContentParser<S>,
    context: &mut Context<'_>,
) -> Result<tree::SectionDefinition<'str>> {
    let kind: Located<S> = attributes.expect_word(context.offsets(), location, "expected section kind")?;

    attributes.expect_end(context);

    match kind.node.deref() {
        "metadata" => {
            let mut metadata = Vec::<Located<tree::MetadataDirective<'str>>>::with_capacity(contents.contents.len());
            for node in contents.contents {
                match node.node.kind.node {
                    structure::NodeKind::Directive(directive) => match directive.deref() {
                        "name" => {
                            let (mut attributes, contents) = parse_node_contents(node.node.contents);

                            let name = attributes.expect_literal_string(context.offsets(), &node.node.kind.offsets);
                            if let Some(name) = context.report_error(name) {
                                let name_offsets = name.offsets.clone();
                                metadata.push(Located::new(
                                    tree::MetadataDirective::Name(Located::new(
                                        il4il::module::ModuleName::<'str>::from_name(
                                            il4il::identifier::Id::from_cow(S::into_cow(name.node.into_contents()))
                                                .expect("TODO: Translate string literal to ID, with escape sequences"),
                                        ),
                                        name_offsets,
                                    )),
                                    node.offsets.start..name.offsets.end,
                                ));
                                attributes.expect_end(context);
                            }

                            contents.expect_empty(context);
                        }
                        _ => {
                            let bad = directive.to_string();
                            context.push_error_at(node.node.kind.offsets, move |f: &mut Formatter| {
                                write!(f, "unknown metadata directive \".{bad}\"")
                            })
                        }
                    },
                    structure::NodeKind::Word(word) => {
                        let word = word.to_string();
                        context.push_error_at(node.node.kind.offsets, move |f: &mut Formatter| {
                            write!(f, "expected metadata directive, but got \"{word}\"")
                        })
                    }
                }
            }

            Ok(tree::SectionDefinition::Metadata(metadata))
        }
        _ => {
            let s = kind.node.to_string();
            Err(Error::new(
                context.offsets().get_location_range(kind.offsets),
                move |f: &mut Formatter| write!(f, "\"{s}\" is not a known section kind"),
            ))
        }
    }
}

pub(super) fn parse<'str, S: StringRef<'str>>(tree: structure::Tree<S>, context: &mut Context<'_>) -> tree::Root<'str> {
    let mut directives = Vec::with_capacity(tree.contents.len());
    for top_node in tree.contents.into_iter() {
        match top_node.node.kind.node {
            structure::NodeKind::Directive(directive) => {
                let (attributes, contents) = parse_node_contents(top_node.node.contents);
                match directive.deref() {
                    "section" => {
                        let r = parse_section(&top_node.node.kind.offsets, attributes, contents, context);
                        if let Some(section) = context.report_error(r) {
                            directives.push(Located::new(tree::TopLevelDirective::Section(section), top_node.offsets))
                        }
                    }
                    _ => {
                        let directive = directive.to_string();
                        context.push_error_at(top_node.node.kind.offsets, move |f: &mut Formatter| {
                            write!(f, "unknown directive \".{directive}\", expected \".section\"")
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
