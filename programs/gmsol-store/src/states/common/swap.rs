use std::collections::HashSet;

use anchor_lang::prelude::*;

use crate::{
    states::{find_market_address, Market},
    CoreError, StoreError,
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

/// Swap params.
#[account(zero_copy)]
#[derive(Default)]
pub struct SwapParamsV2 {
    /// The length of primary swap path.
    primary_length: u8,
    /// The length of secondary swap path.
    secondary_length: u8,
    padding_0: [u8; 2],
    /// Swap paths.
    paths: [Pubkey; 12],
}

impl SwapParamsV2 {
    /// Max total length of swap paths.
    pub const MAX_TOTAL_LENGTH: usize = 12;

    /// Get the length of primary swap path.
    pub fn primary_length(&self) -> usize {
        usize::from(self.primary_length)
    }

    /// Get the length of secondary swap path.
    pub fn secondary_length(&self) -> usize {
        usize::from(self.secondary_length)
    }

    /// Get primary swap path.
    pub fn primary_swap_path(&self) -> &[Pubkey] {
        let end = self.primary_length();
        &self.paths[0..end]
    }

    /// Get secondary swap path.
    pub fn secondary_swap_path(&self) -> &[Pubkey] {
        let start = self.primary_length();
        let end = start.saturating_add(self.secondary_length());
        &self.paths[start..end]
    }

    pub(crate) fn validate_and_init<'info>(
        &mut self,
        primary_length: u8,
        secondary_length: u8,
        paths: &'info [AccountInfo<'info>],
        store: &Pubkey,
        token_ins: (&Pubkey, &Pubkey),
        token_outs: (&Pubkey, &Pubkey),
    ) -> Result<()> {
        let primary_end = usize::from(primary_length);
        let end = primary_end.saturating_add(usize::from(secondary_length));
        require_gte!(
            Self::MAX_TOTAL_LENGTH,
            end,
            CoreError::InvalidSwapPathLength
        );

        require_gte!(paths.len(), end, CoreError::NotEnoughSwapMarkets);
        let primary_markets = &paths[..primary_end];
        let secondary_markets = &paths[primary_end..end];

        let (primary_token_in, secondary_token_in) = token_ins;
        let (primary_token_out, secondary_token_out) = token_outs;

        validate_path(primary_markets, store, primary_token_in, primary_token_out)?;
        validate_path(
            secondary_markets,
            store,
            secondary_token_in,
            secondary_token_out,
        )?;

        self.primary_length = primary_length;
        self.secondary_length = secondary_length;

        for (idx, market) in paths[0..end].iter().enumerate() {
            self.paths[idx] = market.key();
        }
        Ok(())
    }

    /// Iterate over both swap paths, primary path first then secondary path.
    pub fn iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.primary_swap_path()
            .iter()
            .chain(self.secondary_swap_path().iter())
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
        let loaders = unpack_markets(remaining_accounts).collect::<Result<Vec<_>>>()?;
        Ok(loaders)
    }
}

impl From<&SwapParamsV2> for SwapParams {
    fn from(value: &SwapParamsV2) -> Self {
        Self {
            long_token_swap_path: value.primary_swap_path().to_owned(),
            short_token_swap_path: value.secondary_swap_path().to_owned(),
        }
    }
}

fn unpack_markets<'info>(
    path: &'info [AccountInfo<'info>],
) -> impl Iterator<Item = Result<AccountLoader<'info, Market>>> {
    path.iter().map(AccountLoader::try_from)
}

fn validate_path<'info>(
    path: &'info [AccountInfo<'info>],
    store: &Pubkey,
    token_in: &Pubkey,
    token_out: &Pubkey,
) -> Result<()> {
    let mut current = *token_in;
    let mut seen = HashSet::<_>::default();

    for market in unpack_markets(path) {
        let market = market?;

        if !seen.insert(market.key()) {
            return err!(CoreError::InvalidSwapPath);
        }

        let market = market.load()?;
        let meta = market.validated_meta(store)?;
        if current == meta.long_token_mint {
            current = meta.short_token_mint;
        } else if current == meta.short_token_mint {
            current = meta.long_token_mint
        } else {
            return err!(CoreError::InvalidSwapPath);
        }
    }

    require_eq!(current, *token_out, CoreError::InvalidSwapPath);

    Ok(())
}
