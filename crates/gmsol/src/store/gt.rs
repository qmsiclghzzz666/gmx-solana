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
        initial_minting_cost: u128,
        grow_factor: u128,
        grow_step: u64,
        ranks: Vec<u64>,
    ) -> RpcBuilder<C>;

    /// Configurate GT order fee dicounts.
    fn gt_set_order_fee_discount_factors(
        &self,
        store: &Pubkey,
        factors: Vec<u128>,
    ) -> RpcBuilder<C>;

    /// Configurate GT referral rewards
    fn gt_set_referral_reward_factors(&self, store: &Pubkey, factors: Vec<u128>) -> RpcBuilder<C>;
}

impl<C: Deref<Target = impl Signer> + Clone> GTOps<C> for crate::Client<C> {
    fn initialize_gt(
        &self,
        store: &Pubkey,
        decimals: u8,
        initial_minting_cost: u128,
        grow_factor: u128,
        grow_step: u64,
        ranks: Vec<u64>,
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
                initial_minting_cost,
                grow_factor,
                grow_step,
                ranks,
            })
    }

    fn gt_set_order_fee_discount_factors(
        &self,
        store: &Pubkey,
        factors: Vec<u128>,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .accounts(accounts::ConfigurateGT {
                authority: self.payer(),
                store: *store,
            })
            .args(instruction::GtSetOrderFeeDiscountFactors { factors })
    }

    fn gt_set_referral_reward_factors(&self, store: &Pubkey, factors: Vec<u128>) -> RpcBuilder<C> {
        self.data_store_rpc()
            .accounts(accounts::ConfigurateGT {
                authority: self.payer(),
                store: *store,
            })
            .args(instruction::GtSetReferralRewardFactors { factors })
    }
}
