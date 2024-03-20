/// A pool for holding tokens.
pub trait Pool {
    /// Number type of the pool.
    type Num;

    /// Get the mutable reference to the long token amount.
    fn long_token_amount_mut(&mut self) -> &mut Self::Num;

    /// Get the mutable reference to the short token amount.
    fn short_token_amount_mut(&mut self) -> &mut Self::Num;
}
