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

/// Encapsulates all IL4IL playground state.
#[derive(Default)]
#[wasm_bindgen]
pub struct Playground {
    strings: il4il_asm::cache::RcStringCache,
    module: il4il::module::Module<'static>,
}

#[wasm_bindgen]
impl Playground {
    pub fn new() -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        Default::default()
    }

    // TODO: Avoid string copying!
    pub fn assemble(&mut self, input: &str) -> Errors {
        match il4il_asm::assemble(input, &self.strings) {
            Ok(module) => {
                self.module = module;
                Default::default()
            }
            Err(e) => Errors {
                errors: e.into_assembly_error(),
            },
        }
    }
}
