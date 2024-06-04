use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gmx_core::MarketExt;

use crate::{
    states::{common::SwapParams, Market, Oracle},
    DataStoreError, GmxCoreError,
};

#[must_use]
struct SwapUtils<'a, 'info> {
    oracle: &'a Oracle,
    markets: &'info [AccountInfo<'info>],
    mints: &'info [AccountInfo<'info>],
    expected_mints: &'a [Pubkey],
}

impl<'a, 'info> SwapUtils<'a, 'info> {
    fn new(
        oracle: &'a Oracle,
        markets: &'info [AccountInfo<'info>],
        mints: &'info [AccountInfo<'info>],
        expected_mints: &'a [Pubkey],
    ) -> Self {
        Self {
            oracle,
            markets,
            mints,
            expected_mints,
        }
    }

    /// Execute the swaps.
    ///
    /// ## Assumptions
    /// - `token_in_amount` should have been recorded in the first market's balance.
    ///
    /// ## Notes
    /// - The final amount is still recorded in the last market's balance, i.e., not transferred out.
    fn execute(
        self,
        expected_token_out: Pubkey,
        mut token_in: Pubkey,
        token_in_amount: &u64,
    ) -> Result<u64> {
        if *token_in_amount == 0 {
            return Ok(0);
        }
        let mut flags = BTreeSet::default();
        let mut amount = *token_in_amount;
        let last_idx = self.markets.len().saturating_sub(1);
        // Invariant: `token_in_amount` has been record.
        for (idx, market) in self.markets.iter().enumerate() {
            require!(flags.insert(market.key), DataStoreError::InvalidSwapPath);
            require!(market.is_writable, DataStoreError::InvalidSwapPath);
            let mut market = Account::<'info, Market>::try_from(market)?;
            {
                market.validate(&self.oracle.store)?;
                let meta = &market.meta;
                let mut mint = Account::<Mint>::try_from(&self.mints[idx])?;
                require_eq!(
                    meta.market_token_mint,
                    mint.key(),
                    DataStoreError::InvalidSwapPath
                );
                require_eq!(
                    mint.key(),
                    self.expected_mints[idx],
                    DataStoreError::InvalidSwapPath
                );
                require!(
                    meta.long_token_mint != meta.short_token_mint,
                    DataStoreError::InvalidSwapPath
                );
                let (is_token_in_long, token_out) = if token_in == meta.long_token_mint {
                    (true, meta.short_token_mint)
                } else if token_in == meta.short_token_mint {
                    (false, meta.long_token_mint)
                } else {
                    return Err(DataStoreError::InvalidSwapPath.into());
                };
                let prices = gmx_core::action::Prices {
                    index_token_price: self
                        .oracle
                        .primary
                        .get(&meta.index_token_mint)
                        .ok_or(DataStoreError::MissingOracelPrice)?
                        .max
                        .to_unit_price(),
                    long_token_price: self
                        .oracle
                        .primary
                        .get(&meta.long_token_mint)
                        .ok_or(DataStoreError::MissingOracelPrice)?
                        .max
                        .to_unit_price(),
                    short_token_price: self
                        .oracle
                        .primary
                        .get(&meta.short_token_mint)
                        .ok_or(DataStoreError::MissingOracelPrice)?
                        .max
                        .to_unit_price(),
                };
                if idx != 0 {
                    market.record_transferred_in_by_token(&token_in, amount)?;
                }
                let report = market
                    .as_market(&mut mint)
                    .swap(is_token_in_long, amount.into(), prices)
                    .map_err(GmxCoreError::from)?
                    .execute()
                    .map_err(GmxCoreError::from)?;
                token_in = token_out;
                amount = (*report.token_out_amount())
                    .try_into()
                    .map_err(|_| DataStoreError::AmountOverflow)?;
                if idx != last_idx {
                    market.record_transferred_out_by_token(&token_out, amount)?;
                }
                msg!("{:?}", report);
            }
            // `exit` must be called to ensure data is written to the storage.
            market.exit(&crate::ID)?;
        }
        require_eq!(
            token_in,
            expected_token_out,
            DataStoreError::InvalidSwapPath
        );
        Ok(amount)
    }
}

/// Perform swap base on [`SwapParams`].
///
/// Expecting the `remaining_accounts` are of the of the following form:
///
/// `[...long_path_markets, ...short_path_markets, ...long_path_mints, ...short_path_mints]`
///
/// ## Check
/// - All remaining_accounts must contain the most recent state.
///  The `exit` and `reload` functions can be called to synchronize any accounts that might have an unsynchronized state.
/// - The `token_in_amount`s are assumed to be recorded in the first market of each swap path.
///
/// ## Notes
/// - The swap out amounts are still being recorded in the last market of each swap path, i.e., they are not transferred out.
pub(crate) fn unchecked_swap_with_params<'info>(
    oracle: &Oracle,
    params: &SwapParams,
    remaining_accounts: &'info [AccountInfo<'info>],
    expected_token_outs: (Pubkey, Pubkey),
    token_ins: (Option<Pubkey>, Option<Pubkey>),
    token_in_amounts: (u64, u64),
) -> Result<(u64, u64)> {
    require!(
        (token_in_amounts.0 == 0) || token_ins.0.is_some(),
        DataStoreError::AmountNonZeroMissingToken
    );
    require!(
        (token_in_amounts.1 == 0) || token_ins.1.is_some(),
        DataStoreError::AmountNonZeroMissingToken
    );

    let [long_swap_path, short_swap_path, long_swap_path_mints, short_swap_path_mints] =
        params.split_swap_paths(remaining_accounts)?;

    let long_token_out_amount = token_ins
        .0
        .map(|token_in| {
            SwapUtils::new(
                oracle,
                long_swap_path,
                long_swap_path_mints,
                &params.long_token_swap_path,
            )
            .execute(expected_token_outs.0, token_in, &token_in_amounts.0)
        })
        .transpose()?
        .unwrap_or_default();
    let short_token_out_amount = token_ins
        .1
        .map(|token_in| {
            SwapUtils::new(
                oracle,
                short_swap_path,
                short_swap_path_mints,
                &params.short_token_swap_path,
            )
            .execute(expected_token_outs.1, token_in, &token_in_amounts.1)
        })
        .transpose()?
        .unwrap_or_default();
    Ok((long_token_out_amount, short_token_out_amount))
}
