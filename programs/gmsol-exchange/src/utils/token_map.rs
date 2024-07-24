use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use gmsol_store::states::{common::TokenRecord, TokenMapAccess};

use crate::ExchangeError;

/// Collect token records for the give tokens.
pub fn token_records<A: TokenMapAccess>(
    token_map: &A,
    tokens: &BTreeSet<Pubkey>,
) -> Result<Vec<TokenRecord>> {
    tokens
        .iter()
        .map(|token| {
            let config = token_map
                .get(token)
                .ok_or(error!(ExchangeError::ResourceNotFound))?;
            TokenRecord::from_config(*token, config)
        })
        .collect::<Result<Vec<_>>>()
}
