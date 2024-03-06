use anchor_lang::solana_program::pubkey::Pubkey;

/// Generate key with the given prefix and sub-key.
pub fn create_key(prefix: &str, key: &str) -> String {
    let mut ans = String::from(prefix);
    ans.push(':');
    ans.push_str(key);
    ans
}

/// Prefix for price feed keys.
pub const PRICE_FEED: &'static str = "PRICE_FEE";

/// Key for price feed.
pub fn create_price_feed_key(token: &Pubkey) -> String {
    create_key(PRICE_FEED, &token.to_string())
}
