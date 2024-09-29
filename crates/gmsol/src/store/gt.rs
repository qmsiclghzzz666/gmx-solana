use std::ops::Deref;

use crate::utils::RpcBuilder;
use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_store::{accounts, instruction};

/// GT Operations.
pub trait GTOps<C> {
    /// Initialize GT Mint.
    fn initialize_gt(
        &self,
        store: &Pubkey,
        decimals: u8,
        mint_base_value: u128,
        initial_mint_rate_factor: u128,
        decay_factor: u128,
        decay_step: u64,
    ) -> RpcBuilder<C>;
}

impl<C: Deref<Target = impl Signer> + Clone> GTOps<C> for crate::Client<C> {
    fn initialize_gt(
        &self,
        store: &Pubkey,
        decimals: u8,
        mint_base_value: u128,
        initial_mint_rate_factor: u128,
        decay_factor: u128,
        decay_step: u64,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .accounts(accounts::InitializeGT {
                authority: self.payer(),
                store: *store,
                gt_mint: self.find_gt_mint_address(store),
                system_program: system_program::ID,
                token_program: anchor_spl::token_2022::ID,
            })
            .args(instruction::InitializeGt {
                decimals,
                mint_base_value,
                initial_mint_rate_factor,
                decay_factor,
                decay_step,
            })
    }
}
