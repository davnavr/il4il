//! Low-level syntax node parser.

use crate::cache::StringRef;
use crate::error::Error;
use crate::lexer::{self, Token};
use crate::syntax::{structure, Located};
use std::fmt::Formatter;
use std::ops::{Deref, Range};

type AttributeList<S> = Vec<Located<structure::Attribute<S>>>;

type NodeList<S> = Vec<Located<structure::Node<S>>>;

enum ParentContents<S: Deref<Target = str>> {
    Line(AttributeList<S>),
    Blocks(AttributeList<S>, NodeList<S>),
}

struct ParentNode<S: Deref<Target = str>> {
    kind: Located<structure::NodeKind<S>>,
    contents: ParentContents<S>,
}

impl<'str, S: StringRef<'str>> ParentNode<S> {
    fn new(kind: structure::NodeKind<S>, offsets: Range<usize>) -> Self {
        Self {
            kind: Located::new(kind, offsets),
            contents: ParentContents::Line(AttributeList::new()),
        }
    }
}

// struct NestedNode<'src> {
//    kind: Located<structure::NodeKind<'src>>,
//    attributes: NodeList<'src>,
//    contents: AttributeList<'src>,
// }
//
// struct Stack<'src> {
//     nested_nodes: Vec<NestedNode<'src>>,
//     top_level_nodes: NodeList<'src>,
// }

pub(super) fn parse<'str, S: StringRef<'str>>(
    tokens: Vec<(lexer::Token<S>, Range<usize>)>,
    context: &mut crate::parser::Context<'_>,
) -> structure::Tree<S> {
    let mut contents = Vec::new();

    // NOTE: Currently, all nodes that are NOT the top of this stack are expected/guaranteed to be Blocks
    let mut nodes = Vec::<ParentNode<S>>::new();

    for (tok, byte_offsets) in tokens.into_iter() {
        if let Some(parent_node) = nodes.last_mut() {
            match tok {
                Token::Unknown(unknown) => {
                    context.push_error_at(byte_offsets, format!("unexpected '{}'", unknown.deref()));
                }
                Token::Semicolon => match &mut parent_node.contents {
                    ParentContents::Line(attributes) => {
                        let attributes = std::mem::take(attributes);
                        let current_node = nodes.pop().unwrap();
                        let offsets = current_node.kind.offsets.start..byte_offsets.end;
                        let siblings = match nodes.last_mut() {
                            None => &mut contents,
                            Some(ParentNode {
                                contents: ParentContents::Blocks(_, nodes),
                                ..
                            }) => nodes,
                            Some(ParentNode {
                                contents: ParentContents::Line(_),
                                ..
                            }) => unreachable!(),
                        };

                        siblings.push(Located::new(
                            structure::Node {
                                kind: current_node.kind,
                                contents: structure::NodeContents::Line(attributes),
                            },
                            offsets,
                        ));
                    }
                    ParentContents::Blocks { .. } => (),
                },
                Token::OpenBracket => match &mut parent_node.contents {
                    ParentContents::Line(attributes) => {
                        let attributes = std::mem::take(attributes);
                        parent_node.contents = ParentContents::Blocks(attributes, Vec::new());
                    }
                    ParentContents::Blocks(_, _) => context.push_error_at(byte_offsets, "unexpected opening bracket in block"),
                },
                Token::CloseBracket => match &mut parent_node.contents {
                    ParentContents::Blocks(attributes, children) => {
                        let attributes = std::mem::take(attributes);
                        let children = std::mem::take(children);
                        let current_node = nodes.pop().unwrap();
                        let offsets = current_node.kind.offsets.start..byte_offsets.end;
                        let siblings = match nodes.last_mut() {
                            None => &mut contents,
                            Some(ParentNode {
                                contents: ParentContents::Blocks(_, nodes),
                                ..
                            }) => nodes,
                            Some(ParentNode {
                                contents: ParentContents::Line(_),
                                ..
                            }) => unreachable!(),
                        };

                        siblings.push(Located::new(
                            structure::Node {
                                kind: current_node.kind,
                                contents: structure::NodeContents::Block {
                                    attributes,
                                    nodes: children,
                                },
                            },
                            offsets,
                        ))
                    }
                    ParentContents::Line(attributes) => {
                        todo!("handle unexpected closing bracket in line")
                    }
                },
                Token::Word(word) => {
                    let attributes = match &mut parent_node.contents {
                        ParentContents::Line(attrs) | ParentContents::Blocks(attrs, _) => attrs,
                    };

                    attributes.push(Located::new(structure::Attribute::Word(word), byte_offsets));
                }
                Token::String(s) => {
                    let attributes = match &mut parent_node.contents {
                        ParentContents::Line(attrs) | ParentContents::Blocks(attrs, _) => attrs,
                    };

                    attributes.push(Located::new(structure::Attribute::String(s), byte_offsets));
                }
                Token::Directive(name) => {
                    nodes.push(ParentNode::new(structure::NodeKind::Directive(name), byte_offsets));
                }
                _ => todo!("{:?}", tok),
            }
        } else {
            match tok {
                Token::Semicolon => (),
                Token::Directive(name) => {
                    nodes.push(ParentNode::new(structure::NodeKind::Directive(name), byte_offsets));
                }
                _ => {
                    let tok = tok.to_string();
                    context.push_error_at(byte_offsets, move |f: &mut Formatter| {
                        write!(f, "unexpected '{tok}', expected directive")
                    });
                }
            }
        }
    }

    if !nodes.is_empty() {
        let nesting_level = nodes.len();
        let last_location = context.offsets().last_location();
        context.push_error(Error::new(
            Range {
                start: last_location,
                end: last_location,
            },
            move |f: &mut Formatter| write!(f, "expected {nesting_level} closing brackets"),
        ));

        for parent_node in nodes {
            todo!("complete the nodes {:?}", parent_node.kind)
        }
    }

    structure::Tree { contents }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::StringArena;
    use crate::lexer;

    #[test]
    fn directive_test() {
        let strings = StringArena::new();
        let tokens = lexer::tokenize(".example word;\n", &strings).unwrap();
        let mut errors = Vec::new();
        let output = parse(
            tokens.tokens,
            &mut crate::parser::Context {
                offsets: &tokens.offsets,
                errors: &mut errors,
            },
        );

        crate::error::assert_ok(errors.iter());
        assert_eq!(
            output,
            structure::Tree {
                contents: vec![Located::new(
                    structure::Node {
                        kind: Located::new(structure::NodeKind::Directive("example"), 0..8),
                        contents: structure::NodeContents::Line(vec![Located::new(structure::Attribute::Word("word"), 9..13)]),
                    },
                    0..14
                )]
            }
        );
    }
}
