use std::{collections::BTreeSet, ops::Deref};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
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
    ) -> crate::Result<(RpcBuilder<C>, Pubkey)>;
}

impl<C: Deref<Target = impl Signer> + Clone> GlvOps<C> for crate::Client<C> {
    fn initialize_glv(
        &self,
        store: &Pubkey,
        index: u8,
        market_tokens: impl IntoIterator<Item = Pubkey>,
    ) -> crate::Result<(RpcBuilder<C>, Pubkey)> {
        let authority = self.payer();
        let glv_token = self.find_glv_token_address(store, index);
        let glv = self.find_glv_address(&glv_token);
        let market_token_program_id = anchor_spl::token::ID;

        let market_token_addresses = market_tokens.into_iter().collect::<BTreeSet<_>>();

        let markets = market_token_addresses
            .iter()
            .map(|token| AccountMeta::new_readonly(self.find_market_address(store, token), false));

        let market_tokens = market_token_addresses
            .iter()
            .map(|token| AccountMeta::new_readonly(*token, false));

        let market_token_vaults = market_token_addresses.iter().map(|token| {
            let market_token_vault =
                get_associated_token_address_with_program_id(&glv, token, &market_token_program_id);

            AccountMeta::new(market_token_vault, false)
        });

        let rpc = self
            .store_rpc()
            .accounts(accounts::InitializeGlv {
                authority,
                store: *store,
                glv_token,
                glv,
                system_program: system_program::ID,
                token_program: anchor_spl::token_2022::ID,
                market_token_program: market_token_program_id,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::InitalizeGlv {
                index,
                length: market_token_addresses
                    .len()
                    .try_into()
                    .map_err(|_| crate::Error::invalid_argument("too many markets"))?,
            })
            .accounts(
                markets
                    .chain(market_tokens)
                    .chain(market_token_vaults)
                    .collect::<Vec<_>>(),
            );
        Ok((rpc, glv_token))
    }
}
