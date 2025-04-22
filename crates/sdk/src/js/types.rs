#![allow(dead_code)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const STRING_PUBKEY_TY: &'static str = r#"
/**
 * A Base58-encoded string representing a public key.
 */
export type StringPubkey = string;
"#;
