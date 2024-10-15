use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
};
use gmsol_store::{accounts, instruction};

use crate::utils::RpcBuilder;

/// Glv Operations.
pub trait GlvOps<C> {
    /// Initialize GLV.
    fn initialize_glv(
        &self,
        store: &Pubkey,
        index: u8,
        market_tokens: impl IntoIterator<Item = Pubkey>,
    ) -> (RpcBuilder<C>, Pubkey);
}

impl<C: Deref<Target = impl Signer> + Clone> GlvOps<C> for crate::Client<C> {
    fn initialize_glv(
        &self,
        store: &Pubkey,
        index: u8,
        market_tokens: impl IntoIterator<Item = Pubkey>,
    ) -> (RpcBuilder<C>, Pubkey) {
        let authority = self.payer();
        let glv_token = self.find_glv_token_address(store, index);
        let glv = self.find_glv_address(&glv_token);
        let markets = market_tokens
            .into_iter()
            .map(|token| AccountMeta::new_readonly(self.find_market_address(store, &token), false));
        let rpc = self
            .store_rpc()
            .accounts(accounts::InitializeGlv {
                authority,
                store: *store,
                glv_token,
                glv,
                system_program: system_program::ID,
                token_program: anchor_spl::token_2022::ID,
            })
            .args(instruction::InitalizeGlv { index })
            .accounts(markets.collect::<Vec<_>>());
        (rpc, glv_token)
    }
}
