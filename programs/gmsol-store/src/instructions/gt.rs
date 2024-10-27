use anchor_lang::prelude::*;

use crate::{states::Store, utils::internal};

/// The accounts defintions for the `initialize_gt` instruction.
#[derive(Accounts)]
pub struct InitializeGt<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn unchecked_initialize_gt(
    ctx: Context<InitializeGt>,
    decimals: u8,
    initial_minting_cost: u128,
    grow_factor: u128,
    grow_step: u64,
    ranks: &[u64],
) -> Result<()> {
    ctx.accounts.initialize_gt_state(
        decimals,
        initial_minting_cost,
        grow_factor,
        grow_step,
        ranks,
    )?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeGt<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> InitializeGt<'info> {
    fn initialize_gt_state(
        &self,
        decimals: u8,
        initial_minting_cost: u128,
        grow_factor: u128,
        grow_step: u64,
        ranks: &[u64],
    ) -> Result<()> {
        let mut store = self.store.load_mut()?;
        store.gt_mut().init(
            decimals,
            initial_minting_cost,
            grow_factor,
            grow_step,
            ranks,
        )?;
        Ok(())
    }
}

/// The accounts defintions for GT configuration instructions.
#[derive(Accounts)]
pub struct ConfigurateGT<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
}

impl<'info> internal::Authentication<'info> for ConfigurateGT<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// CHECK: only MARKET_KEEPER is authorized to use this instruction.
pub(crate) fn unchecked_gt_set_order_fee_discount_factors(
    ctx: Context<ConfigurateGT>,
    factors: &[u128],
) -> Result<()> {
    ctx.accounts
        .store
        .load_mut()?
        .gt_mut()
        .set_order_fee_discount_factors(factors)
}

/// CHECK: only MARKET_KEEPER is authorized to use this instruction.
pub(crate) fn unchecked_gt_set_referral_reward_factors(
    ctx: Context<ConfigurateGT>,
    factors: &[u128],
) -> Result<()> {
    ctx.accounts
        .store
        .load_mut()?
        .gt_mut()
        .set_referral_reward_factors(factors)
}
