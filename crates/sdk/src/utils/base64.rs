use base64::engine::{general_purpose, Engine};

/// Encode base64
pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64
pub fn decode_base64(data: &str) -> crate::Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(data)?)
}
