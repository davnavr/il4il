//! Low-level syntax node parser.

use crate::error::Error;
use crate::lexer::{self, Token};
use crate::syntax::{structure, Located};
use std::ops::Range;

type AttributeList<'src> = Vec<Located<structure::Attribute<'src>>>;

type NodeList<'src> = Vec<Located<structure::Node<'src>>>;

enum ParentContents<'src> {
    Line(AttributeList<'src>),
    Blocks(AttributeList<'src>, NodeList<'src>),
}

struct ParentNode<'src> {
    kind: Located<structure::NodeKind<'src>>,
    contents: ParentContents<'src>,
}

impl<'src> ParentNode<'src> {
    fn new(kind: structure::NodeKind<'src>, offsets: Range<usize>) -> Self {
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

pub(super) fn parse<'src>(
    tokens: Vec<(lexer::Token<'src>, Range<usize>)>,
    offsets: &lexer::Offsets,
    errors: &mut Vec<Error>,
) -> structure::Tree<'src> {
    let mut contents = Vec::new();

    // NOTE: Currently, all nodes that are NOT the top of this stack are expected/guaranteed to be Blocks
    let mut nodes = Vec::<ParentNode<'src>>::new();

    for (tok, byte_offsets) in tokens.into_iter() {
        if let Some(parent_node) = nodes.last_mut() {
            match tok {
                Token::Unknown(unknown) => {
                    let message = format!("unexpected '{unknown}'");
                    errors.push(Error::new(offsets.get_location_range(byte_offsets), move |f| f.write_str(&message)));
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
                    ParentContents::Blocks(_, _) => errors.push(Error::new(offsets.get_location_range(byte_offsets), |f| {
                        f.write_str("unexpected opening bracket in block")
                    })),
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
                _ => todo!("{:?}", tok),
            }
        } else {
            match tok {
                Token::Semicolon => (),
                Token::Directive(name) => {
                    nodes.push(ParentNode::new(structure::NodeKind::Directive(name), byte_offsets));
                }
                _ => {
                    let message = format!("unexpected '{}', expected directive or word", tok);
                    errors.push(Error::new(offsets.get_location_range(byte_offsets), move |f| f.write_str(&message)));
                }
            }
        }
    }

    if !nodes.is_empty() {
        let nesting_level = nodes.len();
        let last_location = offsets.last_location();
        errors.push(Error::new(
            Range {
                start: last_location,
                end: last_location,
            },
            move |f| write!(f, "expected {nesting_level} closing brackets"),
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
    use crate::cache::StringCache;
    use crate::lexer;

    #[test]
    fn directive_test() {
        let strings = StringCache::new();
        let tokens = lexer::tokenize(".example word;\n", &strings).unwrap();
        let mut errors = Vec::new();
        let output = parse(tokens.tokens, &tokens.offsets, &mut errors);
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
