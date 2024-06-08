use anchor_lang::solana_program::pubkey::Pubkey;

/// Convert to bytes with only the reference of a [`Pubkey`].
pub fn to_bytes(pubkey: &Pubkey) -> [u8; 32] {
    pubkey.to_bytes()
}
