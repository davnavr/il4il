#![doc = include_str!("../README.md")]

use wasm_bindgen::prelude::*;

#[derive(Default)]
#[wasm_bindgen]
pub struct Errors {
    errors: Vec<il4il_asm::error::Error>,
}

#[wasm_bindgen]
impl Errors {
    pub fn count(&self) -> usize {
        self.errors.len()
    }

    pub fn get(&self, index: usize) -> String {
        self.errors.get(index).map(|e| e.to_string()).unwrap_or_default()
    }
}

#[derive(Default)]
pub struct Assembler<'a> {
    string_cache: il4il_asm::cache::StringCache<'a>,
    module: il4il::module::Module<'a>,
}

impl<'a> Assembler<'a> {
    pub fn assemble(&'a mut self, s: &str) -> Errors {
        match il4il_asm::assemble(s, &self.string_cache) {
            Ok(module) => {
                self.module = module;
                Default::default()
            }
            Err(errors) => Errors {
                errors: errors.into_assembly_error(),
            },
        }
    }
}

/// Encapsulates all IL4IL assembler state.
#[wasm_bindgen]
pub struct Playground {
    assembler: Assembler<'static>,
}

#[wasm_bindgen]
impl Playground {
    pub fn new() -> Self {
        Self {
            assembler: Default::default(),
        }
    }

    // TODO: Avoid string copying!
    pub fn assemble(&mut self, s: &str) -> Errors {
        // let e: &mut Assembler<'_> = unsafe { std::mem::transmute(&mut self.assembler) };
        // e.assemble(s)
        Default::default()
    }
}
