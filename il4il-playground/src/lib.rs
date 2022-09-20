#![doc = include_str!("../README.md")]

use wasm_bindgen::prelude::*;
use std::rc::Rc;

#[derive(Default)]
#[wasm_bindgen]
pub struct AssemblerErrors {
    errors: Vec<il4il_asm::error::Error>,
}

/// Encapsulates all IL4IL assembler state.
#[wasm_bindgen]
pub struct Playground {
    string_cache: Rc<il4il_asm::cache::StringCache<'static>>,
    module: Option<Rc<il4il::module::Module<'static>>>,
}

#[wasm_bindgen]
impl Playground {
    pub fn new() -> Self {
        Self {
            string_cache: Default::default(),
            module: None,
        }
    }

    // TODO: Avoid string copying!
    pub fn assemble(&mut self, s: &str) -> AssemblerErrors {
        match il4il_asm::assemble(s, &self.string_cache) {
            Ok(module) => {
                self.module = Some(Rc::new(module));
                Default::default()
            }
            Err(errors) => {
                AssemblerErrors {
                    errors: errors.into_assembly_error(),
                }
            }
        }
    }
}
