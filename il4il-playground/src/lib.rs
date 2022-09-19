#![doc = include_str!("../README.md")]
#![cfg(target_family = "wasm")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub extern "C" fn do_something(a: i32) -> i32 {
    a + 5
}
