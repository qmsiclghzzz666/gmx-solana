use std::collections::HashSet;

use anchor_lang::prelude::*;

use crate::CoreError;

use super::Seed;

const MAX_ALLOWED_NUMBER_OF_MARKETS: usize = 128;

/// Glv.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Glv {
    version: u8,
    /// Bump seed.
    pub(crate) bump: u8,
    /// Index.
    pub(crate) index: u8,
    /// Num of markets.
    pub(crate) num_markets: u8,
    padding: [u8; 4],
    pub(crate) store: Pubkey,
    pub(crate) glv_token: Pubkey,
    pub(crate) long_token: Pubkey,
    pub(crate) short_token: Pubkey,
    reserve: [u8; 256],
    market_tokens: [Pubkey; MAX_ALLOWED_NUMBER_OF_MARKETS],
}

impl Seed for Glv {
    const SEED: &'static [u8] = b"glv";
}

impl Glv {
    /// Init space.
    pub const INIT_SPACE: usize = std::mem::size_of::<Self>();

    /// GLV token seed.
    pub const GLV_TOKEN_SEED: &'static [u8] = b"glv_token";

    /// Initialize the [`Glv`] account.
    ///
    /// # CHECK
    /// - The [`Glv`] account must be uninitialized.
    /// - The `bump` must be the bump derving the address of the [`Glv`] account.
    /// - The `glv_token` must be used to dervie the address of the [`Glv`] account.
    /// - The market tokens must be valid, and their corresponding markets
    ///   must use the given tokens as long token and short token.
    /// - The `store` must be the address of the store owning the correspoding markets.
    ///
    /// # Errors
    /// - The `glv_token` address must be derived from [`GLV_TOKEN_SEED`](Self::GLV_TOKEN_SEED), `store` and `index`.
    /// - The total number of the market tokens must not exceed the max allowed number of markets.
    /// - The market tokens must be unique.
    pub(crate) fn unchecked_init<'a>(
        &mut self,
        bump: u8,
        index: u8,
        store: &Pubkey,
        glv_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
        market_tokens: impl IntoIterator<Item = &'a Pubkey>,
    ) -> Result<()> {
        let expected_glv_token = Pubkey::find_program_address(
            &[Self::GLV_TOKEN_SEED, store.as_ref(), &[index]],
            &crate::ID,
        )
        .0;
        require_eq!(expected_glv_token, *glv_token, CoreError::InvalidArgument);

        self.version = 0;
        self.bump = bump;
        self.index = index;
        self.store = *store;
        self.glv_token = *glv_token;
        self.long_token = *long_token;
        self.short_token = *short_token;

        let mut seen = HashSet::<_>::default();
        for (idx, market_token) in market_tokens.into_iter().enumerate() {
            require!(seen.insert(market_token), CoreError::InvalidArgument);
            self.num_markets += 1;
            require_gte!(
                MAX_ALLOWED_NUMBER_OF_MARKETS,
                self.num_markets as usize,
                CoreError::ExceedMaxLengthLimit
            );
            self.market_tokens[idx] = *market_token;
        }
        Ok(())
    }

    /// Get the version of the [`Glv`] account format.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get the index of the glv token.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Get the store address.
    pub fn store(&self) -> &Pubkey {
        &self.store
    }

    /// Get the GLV token address.
    pub fn glv_token(&self) -> &Pubkey {
        &self.glv_token
    }

    /// Get the long token address.
    pub fn long_token(&self) -> &Pubkey {
        &self.long_token
    }

    /// Get the short token address.
    pub fn short_token(&self) -> &Pubkey {
        &self.short_token
    }

    /// Get all market tokens.
    pub fn market_tokens(&self) -> &[Pubkey] {
        &self.market_tokens[0..(self.num_markets as usize)]
    }
}
