use anchor_lang::{err, error, require, Result};

use crate::DataStoreError;

/// Fixed size string to bytes.
pub fn fixed_str_to_bytes<const MAX_LEN: usize>(name: &str) -> Result<[u8; MAX_LEN]> {
    let bytes = name.as_bytes();
    require!(
        bytes.len() <= MAX_LEN,
        DataStoreError::ExceedMaxStringLengthLimit
    );
    let mut buffer = [0; MAX_LEN];
    buffer[..bytes.len()].copy_from_slice(bytes);
    Ok(buffer)
}

/// Bytes to fixed size string.
pub fn bytes_to_fixed_str(bytes: &[u8; 32]) -> Result<&str> {
    let Some(end) = bytes.iter().position(|&x| x == 0) else {
        return err!(DataStoreError::InvalidArgument);
    };
    let valid_bytes = &bytes[..end];
    std::str::from_utf8(valid_bytes).map_err(|_| error!(DataStoreError::InvalidArgument))
}
