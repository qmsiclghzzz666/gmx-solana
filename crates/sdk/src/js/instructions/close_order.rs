use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    builders::{
        order::{CloseOrder, CloseOrderHint},
        token::PrepareTokenAccounts,
        StoreProgram,
    },
    utils::serde::StringPubkey,
};

use super::{TransactionGroup, TransactionGroupOptions};
use gmsol_solana_utils::{signer::TransactionSigners, IntoAtomicGroup};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::NullSigner;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

/// Parameters for closing orders.
#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CloseOrderParams {
    recent_blockhash: String,
    payer: StringPubkey,
    orders: HashMap<StringPubkey, CloseOrderHint>,
    #[serde(default)]
    program: Option<StoreProgram>,
    #[serde(default)]
    transaction_group: TransactionGroupOptions,
}

/// Build transactions for closing orders.
#[wasm_bindgen]
pub fn close_orders(params: CloseOrderParams) -> crate::Result<TransactionGroup> {
    let mut group = params.transaction_group.build();

    let payer = params.payer;
    let program = params.program.unwrap_or_default();
    let mut tokens = HashMap::<_, HashSet<_>>::default();

    for hint in params.orders.values() {
        let owner = hint.owner;
        let owner_tokens = tokens.entry(owner).or_default();
        if let Some(token) = hint.initial_collateral_token {
            owner_tokens.insert(token);
        }

        let receiver = hint.receiver;
        let receiver_tokens = tokens.entry(receiver).or_default();
        if let Some(token) = hint.final_output_token {
            receiver_tokens.insert(token);
        }
        if let Some(token) = hint.long_token {
            receiver_tokens.insert(token);
        }
        if let Some(token) = hint.short_token {
            receiver_tokens.insert(token);
        }
    }

    for (owner, tokens) in tokens {
        group.add(
            PrepareTokenAccounts::builder()
                .owner(owner)
                .payer(payer)
                .tokens(tokens)
                .build()
                .into_atomic_group(&())?,
        )?;
    }

    for (order, hint) in params.orders {
        group.add(
            CloseOrder::builder()
                .payer(payer)
                .order(order)
                .program(program.clone())
                .build()
                .into_atomic_group(&hint)?,
        )?;
    }

    let signers = TransactionSigners::<Arc<NullSigner>>::default();
    let transactions = group
        .optimize(false)
        .to_transactions(
            &signers,
            params
                .recent_blockhash
                .parse()
                .map_err(crate::Error::unknown)?,
            true,
        )
        .map(|res| res.map_err(crate::Error::from))
        .collect::<crate::Result<Vec<_>>>()?;

    Ok(TransactionGroup(transactions))
}
