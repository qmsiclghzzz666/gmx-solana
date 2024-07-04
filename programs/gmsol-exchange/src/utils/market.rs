use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use gmsol_store::{
    cpi::accounts::{GetValidatedMarketMeta, ReadMarket},
    program::GmsolStore,
};

use crate::ExchangeError;

pub(crate) fn get_and_validate_swap_path<'info>(
    program: &Program<'info, GmsolStore>,
    store: AccountInfo<'info>,
    accounts: &[AccountInfo<'info>],
    initial_token: &Pubkey,
    final_token: &Pubkey,
    tokens: &mut BTreeSet<Pubkey>,
) -> Result<Vec<Pubkey>> {
    let mut current = *initial_token;
    let mut flags = BTreeSet::default();
    let markets = accounts
        .iter()
        .map(|account| {
            if !flags.insert(account.key) {
                return Err(ExchangeError::InvalidSwapPath.into());
            }
            let meta = gmsol_store::cpi::get_validated_market_meta(CpiContext::new(
                program.to_account_info(),
                GetValidatedMarketMeta {
                    store: store.clone(),
                    market: account.clone(),
                },
            ))?
            .get();
            if meta.long_token_mint == meta.short_token_mint {
                return Err(ExchangeError::InvalidSwapPath.into());
            }
            if current == meta.long_token_mint {
                current = meta.short_token_mint;
            } else if current == meta.short_token_mint {
                current = meta.long_token_mint;
            } else {
                return Err(ExchangeError::InvalidSwapPath.into());
            }
            tokens.insert(meta.index_token_mint);
            tokens.insert(meta.long_token_mint);
            tokens.insert(meta.short_token_mint);
            Ok(meta.market_token_mint)
        })
        .collect::<Result<Vec<_>>>()?;
    require_eq!(current, *final_token, ExchangeError::InvalidSwapPath);
    Ok(markets)
}

pub(crate) fn get_market_token_mint<'info>(
    data_store_program: &AccountInfo<'info>,
    market: &AccountInfo<'info>,
) -> Result<Pubkey> {
    let meta = gmsol_store::cpi::get_market_meta(CpiContext::new(
        data_store_program.clone(),
        ReadMarket {
            market: market.clone(),
        },
    ))?
    .get();
    Ok(meta.market_token_mint)
}
