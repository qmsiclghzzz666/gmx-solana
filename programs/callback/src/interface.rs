use anchor_lang::{prelude::Pubkey, pubkey, Ids};

pub use crate::{
    cpi::{accounts::OnCallback, on_closed, on_created, on_executed},
    types::ActionKind,
    CALLBACK_AUTHORITY_SEED,
};

#[cfg(not(feature = "no-competition"))]
const COMPETITION_ID: Pubkey = pubkey!("2AxuNr6euZPKQbTwNsLBjzFTZFAevA85F4PW9m9Dv8pc");

/// Callback interface for GMX-Solana.
#[derive(Debug, Clone, Copy, Default)]
pub struct CallbackInterface;

impl Ids for CallbackInterface {
    fn ids() -> &'static [Pubkey] {
        static IDS: &[Pubkey] = &[
            #[cfg(feature = "test-only")]
            crate::ID,
            #[cfg(not(feature = "no-competition"))]
            COMPETITION_ID,
        ];

        IDS
    }
}
