//! Representation of IL4IL function bodies.

use crate::module::Module;
use crate::types;
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

type CodeBlocks<'env> = lazy_init::LazyTransform<(il4il::instruction::Block, Box<[il4il::instruction::Block]>), Box<[Block<'env>]>>;

/// Represents an IL4IL function body.
pub struct Code<'env> {
    module: &'env Module<'env>,
    index: index::FunctionBody,
    result_types: types::ReferenceList<'env>,
    blocks: CodeBlocks<'env>,
}

impl<'env> Code<'env> {
    pub(crate) fn new(module: &'env Module<'env>, index: index::FunctionBody, code: il4il::function::Body) -> Self {
        Self {
            module,
            index,
            result_types: types::ReferenceList::new(module, code.result_types),
            blocks: CodeBlocks::new((code.entry_block, code.other_blocks)),
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
        self.blocks.get_or_create(|(entry_block, other_blocks)| {
            let mut blocks = Vec::with_capacity(other_blocks.len() + 1);
            blocks.push(Block::new(self, 0.into(), entry_block));
            blocks.extend(
                other_blocks
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

    pub fn result_types(&'env self) -> &'env [types::Reference<'env>] {
        self.result_types.types()
    }
}

impl Debug for Code<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code").finish_non_exhaustive()
    }
}
