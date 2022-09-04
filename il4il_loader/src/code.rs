//! Representation of IL4IL function bodies.

use crate::module::Module;
use il4il::index;
use std::fmt::{Debug, Formatter};

pub struct Block<'env> {
    body: &'env Code<'env>,
    index: index::Block,
    instructions: Vec<il4il::instruction::Instruction>,
}

impl<'env> Block<'env> {
    fn new(body: &'env Code<'env>, index: index::Block, block: il4il::instruction::Block) -> Self {
        Self {
            body,
            index,
            instructions: block.instructions,
        }
    }

    pub fn index(&'env self) -> index::Block {
        self.index
    }

    pub fn body(&'env self) -> &'env Code<'env> {
        self.body
    }

    pub fn module(&'env self) -> &'env Module {
        self.body.module()
    }

    pub fn instructions(&'env self) -> &'env [il4il::instruction::Instruction] {
        &self.instructions
    }
}

type CodeBlocks<'env> = lazy_init::LazyTransform<il4il::function::Body, Box<[Block<'env>]>>;

pub struct Code<'env> {
    module: &'env Module<'env>,
    index: index::FunctionBody,
    blocks: CodeBlocks<'env>,
}

impl<'env> Code<'env> {
    pub(crate) fn new(module: &'env Module<'env>, index: index::FunctionBody, code: il4il::function::Body) -> Self {
        Self {
            module,
            index,
            blocks: CodeBlocks::new(code),
        }
    }

    pub fn module(&'env self) -> &'env Module<'env> {
        self.module
    }

    pub fn index(&'env self) -> index::FunctionBody {
        self.index
    }

    /// Returns the function body's basic blocks.
    pub fn blocks(&'env self) -> &'env [Block<'env>] {
        self.blocks.get_or_create(|body| {
            let mut blocks = Vec::with_capacity(body.other_blocks().len() + 1);
            blocks.push(Block::new(self, 0.into(), body.entry_block));
            blocks.extend(
                body.other_blocks
                    .into_vec()
                    .into_iter()
                    .enumerate()
                    .map(|(index, block)| Block::new(self, (index + 1).into(), block)),
            );
            blocks.into_boxed_slice()
        })
    }

    pub fn entry_block(&'env self) -> &'env Block<'env> {
        self.blocks().first().expect("entry block should always exist")
    }
}

impl Debug for Code<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code").finish_non_exhaustive()
    }
}
