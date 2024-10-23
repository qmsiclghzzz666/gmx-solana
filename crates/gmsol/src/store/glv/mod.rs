use std::{collections::BTreeSet, ops::Deref};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use gmsol_store::{accounts, instruction, states::Market};

use crate::utils::RpcBuilder;

mod deposit;

pub use self::deposit::{
    CloseGlvDepositBuilder, CloseGlvDepositHint, CreateGlvDepositBuilder, CreateGlvDepositHint,
};

/// Glv Operations.
pub trait GlvOps<C> {
    /// Initialize GLV.
    fn initialize_glv(
        &self,
        store: &Pubkey,
        index: u8,
        market_tokens: impl IntoIterator<Item = Pubkey>,
    ) -> crate::Result<(RpcBuilder<C>, Pubkey)>;

    /// Create a GLV deposit.
    fn create_glv_deposit(
        &self,
        store: &Pubkey,
        glv_token: &Pubkey,
        market_token: &Pubkey,
    ) -> CreateGlvDepositBuilder<C>;

    /// Close a GLV deposit.
    fn close_glv_deposit(&self, glv_deposit: &Pubkey) -> CloseGlvDepositBuilder<C>;
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

        let (accounts, length) = split_to_accounts(
            market_tokens,
            &glv,
            store,
            &self.store_program_id(),
            &market_token_program_id,
        );

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
                length: length
                    .try_into()
                    .map_err(|_| crate::Error::invalid_argument("too many markets"))?,
            })
            .accounts(accounts);
        Ok((rpc, glv_token))
    }

    fn create_glv_deposit(
        &self,
        store: &Pubkey,
        glv_token: &Pubkey,
        market_token: &Pubkey,
    ) -> CreateGlvDepositBuilder<C> {
        CreateGlvDepositBuilder::new(self, *store, *glv_token, *market_token)
    }

    fn close_glv_deposit(&self, glv_deposit: &Pubkey) -> CloseGlvDepositBuilder<C> {
        CloseGlvDepositBuilder::new(self, *glv_deposit)
    }
}

fn split_to_accounts(
    market_tokens: impl IntoIterator<Item = Pubkey>,
    glv: &Pubkey,
    store: &Pubkey,
    store_program_id: &Pubkey,
    token_program_id: &Pubkey,
) -> (Vec<AccountMeta>, usize) {
    let market_token_addresses = market_tokens.into_iter().collect::<BTreeSet<_>>();

    let markets = market_token_addresses.iter().map(|token| {
        AccountMeta::new_readonly(
            Market::find_market_address(store, token, store_program_id).0,
            false,
        )
    });

    let market_tokens = market_token_addresses
        .iter()
        .map(|token| AccountMeta::new_readonly(*token, false));

    let market_token_vaults = market_token_addresses.iter().map(|token| {
        let market_token_vault =
            get_associated_token_address_with_program_id(glv, token, token_program_id);

        AccountMeta::new(market_token_vault, false)
    });

    let length = market_token_addresses.len();
    let accounts = markets
        .chain(market_tokens)
        .chain(market_token_vaults)
        .collect::<Vec<_>>();

    (accounts, length)
}
