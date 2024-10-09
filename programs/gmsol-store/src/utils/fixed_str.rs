use anchor_lang::{err, error, require, Result};

use crate::CoreError;

/// Fixed size string to bytes.
pub fn fixed_str_to_bytes<const MAX_LEN: usize>(name: &str) -> Result<[u8; MAX_LEN]> {
    let bytes = name.as_bytes();
    require!(bytes.len() <= MAX_LEN, CoreError::ExceedMaxLengthLimit);
    let mut buffer = [0; MAX_LEN];
    buffer[..bytes.len()].copy_from_slice(bytes);
    Ok(buffer)
}

/// Bytes to fixed size string.
pub fn bytes_to_fixed_str<const MAX_LEN: usize>(bytes: &[u8; MAX_LEN]) -> Result<&str> {
    let Some(end) = bytes.iter().position(|&x| x == 0) else {
        return err!(CoreError::InvalidArgument);
    };
    let valid_bytes = &bytes[..end];
    std::str::from_utf8(valid_bytes).map_err(|_| error!(CoreError::InvalidArgument))
}
