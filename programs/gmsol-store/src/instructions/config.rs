use anchor_lang::prelude::*;

use crate::{
    states::{Amount, Factor, Store},
    utils::internal,
};

#[derive(Accounts)]
pub struct InsertAmount<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut)]
    store: AccountLoader<'info, Store>,
}

/// Insert amount.
pub fn insert_amount(ctx: Context<InsertAmount>, key: &str, amount: Amount) -> Result<()> {
    *ctx.accounts.store.load_mut()?.get_amount_mut(key)? = amount;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertAmount<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertFactor<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut)]
    store: AccountLoader<'info, Store>,
}

/// Insert factor.
pub fn insert_factor(ctx: Context<InsertFactor>, key: &str, factor: Factor) -> Result<()> {
    *ctx.accounts.store.load_mut()?.get_factor_mut(key)? = factor;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertFactor<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertAddress<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut)]
    store: AccountLoader<'info, Store>,
}

/// Insert address.
pub fn insert_address(ctx: Context<InsertAddress>, key: &str, address: Pubkey) -> Result<()> {
    *ctx.accounts.store.load_mut()?.get_address_mut(key)? = address;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertAddress<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
