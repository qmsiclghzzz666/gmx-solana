use std::collections::HashSet;

use anchor_lang::prelude::*;

use crate::{
    states::{find_market_address, Market},
    StoreError,
};

/// Swap params.
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SwapParams {
    /// The addresses of token mints for markets along the swap path for long token or primary token.
    ///
    /// Market addresses are not cached as they can be derived
    /// by seeding with the corresponding mint addresses.
    pub long_token_swap_path: Vec<Pubkey>,
    /// The addresses of token mints for markets along the swap path for short token or secondary token.
    ///
    /// Market addresses are not cached as they can be derived
    /// by seeding with the corresponding mint addresses.
    pub short_token_swap_path: Vec<Pubkey>,
}

impl SwapParams {
    pub(crate) fn init_space(long_path_len: usize, short_path_len: usize) -> usize {
        (4 + 32 * long_path_len) + (4 + 32 * short_path_len)
    }

    /// Get the first market token in the swap path.
    pub fn first_market_token(&self, is_long: bool) -> Option<&Pubkey> {
        if is_long {
            self.long_token_swap_path.first()
        } else {
            self.short_token_swap_path.first()
        }
    }

    /// Get the last market token in the swap path.
    pub fn last_market_token(&self, is_long: bool) -> Option<&Pubkey> {
        if is_long {
            self.long_token_swap_path.last()
        } else {
            self.short_token_swap_path.last()
        }
    }

    /// Iterate over both swap paths, long path first then short path.
    pub fn iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.long_token_swap_path
            .iter()
            .chain(self.short_token_swap_path.iter())
    }

    /// Split swap paths from accounts.
    ///
    /// Return `[long_path_markets, short_path_markets, long_path_mints, short_path_mints]`.
    pub fn split_swap_paths<'a, 'info>(
        &self,
        remaining_accounts: &'a [AccountInfo<'info>],
    ) -> Result<[&'a [AccountInfo<'info>]; 4]> {
        let long_len = self.long_token_swap_path.len();
        let total_len = self.long_token_swap_path.len() + self.short_token_swap_path.len();

        require_gte!(
            remaining_accounts.len(),
            total_len * 2,
            ErrorCode::AccountNotEnoughKeys
        );

        let long_swap_path = &remaining_accounts[0..long_len];
        let short_swap_path = &remaining_accounts[long_len..total_len];

        let remaining_accounts = &remaining_accounts[total_len..];
        let long_swap_path_mints = &remaining_accounts[0..long_len];
        let short_swap_path_mints = &remaining_accounts[long_len..total_len];

        Ok([
            long_swap_path,
            short_swap_path,
            long_swap_path_mints,
            short_swap_path_mints,
        ])
    }

    /// Get unique market tokens excluding current market token.
    pub fn unique_market_tokens_excluding_current<'a>(
        &'a self,
        current_market_token: &'a Pubkey,
    ) -> impl Iterator<Item = &Pubkey> + 'a {
        let mut seen = HashSet::from([current_market_token]);
        self.iter().filter(move |token| seen.insert(token))
    }

    /// Unpack markets for swap.
    pub fn unpack_markets_for_swap<'info>(
        &self,
        current_market_token: &Pubkey,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<Vec<AccountLoader<'info, Market>>> {
        let len = self
            .unique_market_tokens_excluding_current(current_market_token)
            .count();
        require_gte!(
            remaining_accounts.len(),
            len,
            ErrorCode::AccountNotEnoughKeys
        );
        let loaders = remaining_accounts
            .iter()
            .take(len)
            .map(AccountLoader::<'info, Market>::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(loaders)
    }

    /// Find last market.
    pub fn find_last_market<'info>(
        &self,
        store: &Pubkey,
        is_long: bool,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Option<AccountInfo<'info>> {
        let path = if is_long {
            &self.long_token_swap_path
        } else {
            &self.short_token_swap_path
        };
        let target = find_market_address(store, path.last()?, &crate::ID).0;
        let info = remaining_accounts.iter().find(|info| *info.key == target)?;
        Some(info.clone())
    }

    /// Get validated long path.
    pub fn validated_long_path(&self) -> Result<&[Pubkey]> {
        let mut seen: HashSet<&Pubkey> = HashSet::default();
        require!(
            self.long_token_swap_path
                .iter()
                .all(move |token| seen.insert(token)),
            StoreError::InvalidSwapPath
        );
        Ok(&self.long_token_swap_path)
    }

    /// Get validated short path.
    pub fn validated_short_path(&self) -> Result<&[Pubkey]> {
        let mut seen: HashSet<&Pubkey> = HashSet::default();
        require!(
            self.short_token_swap_path
                .iter()
                .all(move |token| seen.insert(token)),
            StoreError::InvalidSwapPath
        );
        Ok(&self.short_token_swap_path)
    }
}
