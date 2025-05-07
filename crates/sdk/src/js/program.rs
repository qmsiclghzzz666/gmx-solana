use wasm_bindgen::prelude::*;

use crate::builders::StoreProgram;

/// Get default [`StoreProgram`].
#[wasm_bindgen]
pub fn default_store_program() -> StoreProgram {
    StoreProgram::default()
}
