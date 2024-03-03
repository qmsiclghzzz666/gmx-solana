/// Convert a string to a seed.
pub fn to_seed(key: &str) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;
    hash(key.as_bytes()).to_bytes()
}
