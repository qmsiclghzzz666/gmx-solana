use anchor_lang::prelude::*;

use crate::{constants, CoreError};

use super::{user::UserHeader, Seed};

const MAX_RANK: usize = 15;
const MAX_FLAGS: usize = 8;

#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct GtState {
    decimals: u8,
    padding_0: [u8; 7],
    /* States */
    pub(crate) last_minted_at: i64,
    total_minted: u64,
    grow_step_amount: u64,
    grow_steps: u64,
    supply: u64,
    es_supply: u64,
    es_vault: u64,
    es_factor: u128,
    /* Configs */
    minting_cost_grow_factor: u128,
    minting_cost: u128,
    es_receiver_factor: u128,
    es_time_window: u32,
    padding_1: [u8; 12],
    max_rank: u64,
    ranks: [u64; MAX_RANK],
    order_fee_discount_factors: [u128; MAX_RANK + 1],
    referral_reward_factors: [u128; MAX_RANK + 1],
    reserved_1: [u8; 256],
}

impl GtState {
    pub(crate) fn init(
        &mut self,
        decimals: u8,
        initial_minting_cost: u128,
        grow_factor: u128,
        grow_step: u64,
        ranks: &[u64],
    ) -> Result<()> {
        require_eq!(self.last_minted_at, 0, CoreError::GTStateHasBeenInitialized);
        require_eq!(self.total_minted, 0, CoreError::GTStateHasBeenInitialized);
        require_eq!(self.grow_steps, 0, CoreError::GTStateHasBeenInitialized);
        require_eq!(self.supply, 0, CoreError::GTStateHasBeenInitialized);
        require_eq!(self.es_supply, 0, CoreError::GTStateHasBeenInitialized);
        require_eq!(self.es_factor, 0, CoreError::GTStateHasBeenInitialized);

        require!(grow_step != 0, CoreError::InvalidGTConfig);

        let max_rank = ranks.len().min(MAX_RANK);
        let ranks = &ranks[0..max_rank];

        // Ranks must be storted.
        require!(
            ranks.windows(2).all(|ab| {
                if let [a, b] = &ab {
                    a < b
                } else {
                    false
                }
            }),
            CoreError::InvalidGTConfig
        );

        let clock = Clock::get()?;

        self.decimals = decimals;
        self.last_minted_at = clock.unix_timestamp;
        self.grow_step_amount = grow_step;
        self.minting_cost_grow_factor = grow_factor;
        self.minting_cost = initial_minting_cost;

        let target = &mut self.ranks[0..max_rank];
        target.copy_from_slice(ranks);
        self.max_rank = max_rank as u64;

        self.es_time_window = constants::DEFAULT_GT_VAULT_TIME_WINDOW;

        Ok(())
    }

    pub(crate) fn set_order_fee_discount_factors(&mut self, factors: &[u128]) -> Result<()> {
        require_eq!(
            factors.len(),
            (self.max_rank + 1) as usize,
            CoreError::InvalidArgument
        );

        require!(
            factors
                .iter()
                .all(|factor| *factor <= constants::MARKET_USD_UNIT),
            CoreError::InvalidArgument
        );

        let target = &mut self.order_fee_discount_factors[0..factors.len()];
        target.copy_from_slice(factors);

        Ok(())
    }

    pub(crate) fn set_referral_reward_factors(&mut self, factors: &[u128]) -> Result<()> {
        require_eq!(
            factors.len(),
            (self.max_rank + 1) as usize,
            CoreError::InvalidArgument
        );

        let target = &mut self.referral_reward_factors[0..factors.len()];
        target.copy_from_slice(factors);

        Ok(())
    }

    pub(crate) fn order_fee_discount_factor(&self, rank: u8) -> Result<u128> {
        require_gte!(self.max_rank, rank as u64, CoreError::InvalidArgument);
        Ok(self.order_fee_discount_factors[rank as usize])
    }

    pub(crate) fn referral_reward_factor(&self, rank: u8) -> Result<u128> {
        require_gte!(self.max_rank, rank as u64, CoreError::InvalidArgument);
        Ok(self.referral_reward_factors[rank as usize])
    }

    pub(crate) fn set_es_recevier_factor(&mut self, factor: u128) -> Result<()> {
        require_gte!(
            constants::MARKET_USD_UNIT,
            factor,
            CoreError::InvalidArgument
        );
        self.es_receiver_factor = factor;
        Ok(())
    }

    pub(crate) fn es_receiver_factor(&self) -> u128 {
        self.es_receiver_factor
    }

    fn next_minting_cost(&self, next_minted: u64) -> Result<Option<(u64, u128)>> {
        use gmsol_model::utils::apply_factor;

        require!(self.grow_step_amount != 0, CoreError::InvalidGTConfig);
        let new_steps = next_minted / self.grow_step_amount;

        if new_steps != self.grow_steps {
            let mut minting_cost = self.minting_cost;
            for _ in self.grow_steps..new_steps {
                minting_cost = apply_factor::<_, { constants::MARKET_DECIMALS }>(
                    &minting_cost,
                    &self.minting_cost_grow_factor,
                )
                .ok_or(error!(CoreError::Internal))?;
            }
            Ok(Some((new_steps, minting_cost)))
        } else {
            Ok(None)
        }
    }

    /// Sync the esGT factor and ming esGT to the given user.
    ///
    /// # CHECK
    /// - `user` must be owned by this store.
    pub(crate) fn unchecked_sync_es_factor(&mut self, user: &mut UserHeader) -> Result<()> {
        use gmsol_model::utils::apply_factor;

        let gt_amount = u128::from(user.gt.amount());

        if gt_amount == 0 {
            return Ok(());
        }

        let current_factor = self.es_factor;
        let user_factor = user.gt.es_factor;
        require_gte!(current_factor, user_factor, CoreError::Internal);

        if current_factor == user_factor {
            return Ok(());
        }

        let diff_factor = current_factor
            .checked_sub(user_factor)
            .ok_or(error!(CoreError::ValueOverflow))?;

        let amount = apply_factor::<_, { constants::MARKET_DECIMALS }>(&gt_amount, &diff_factor)
            .ok_or(error!(CoreError::ValueOverflow))?;

        let next_es_amount = user
            .gt
            .es_amount
            .checked_add(amount.try_into()?)
            .ok_or(error!(CoreError::TokenAmountOverflow))?;

        /* The following steps should be infallible. */

        user.gt.es_amount = next_es_amount;
        user.gt.es_factor = current_factor;

        Ok(())
    }

    #[inline(never)]
    pub(crate) fn mint_to(&mut self, user: &mut UserHeader, amount: u64) -> Result<()> {
        if amount != 0 {
            let clock = Clock::get()?;

            // Calculate global GT state updates.
            let next_gt_total_minted = self
                .total_minted
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
            let next_minting_cost = self.next_minting_cost(next_gt_total_minted)?;

            // Calculate user GT state updates.
            let next_user_total_minted = user
                .gt
                .total_minted
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
            let next_amount = user
                .gt
                .amount
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
            let next_supply = self
                .supply
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;

            self.unchecked_sync_es_factor(user)?;

            /* The following steps should be infallible. */

            if let Some((new_steps, new_minting_cost)) = next_minting_cost {
                self.minting_cost = new_minting_cost;
                self.grow_steps = new_steps;
            }
            self.total_minted = next_gt_total_minted;
            self.last_minted_at = clock.unix_timestamp;

            user.gt.total_minted = next_user_total_minted;
            user.gt.amount = next_amount;
            user.gt.last_minted_at = self.last_minted_at;
            self.supply = next_supply;
        }
        Ok(())
    }

    /// Burn GT from the given `user`.
    ///
    /// # CHECK
    /// - The `user` must be owned by this store.
    ///
    /// # Errors
    /// - `user` must have enough amount of GT.
    pub(crate) fn unchecked_burn_from(&mut self, user: &mut UserHeader, amount: u64) -> Result<()> {
        if amount != 0 {
            require_gte!(user.gt.amount, amount, CoreError::NotEnoughTokenAmount);
            let next_amount = user
                .gt
                .amount
                .checked_sub(amount)
                .ok_or(error!(CoreError::Internal))?;
            let next_supply = self
                .supply
                .checked_sub(amount)
                .ok_or(error!(CoreError::Internal))?;

            self.unchecked_sync_es_factor(user)?;

            /* The following steps should be infallible. */

            user.gt.amount = next_amount;
            self.supply = next_supply;
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn get_mint_amount(
        &self,
        size_in_value: u128,
        discount: u128,
    ) -> Result<(u64, u128)> {
        use gmsol_model::utils::apply_factor;

        // Calculate the minting cost to apply.
        let minting_cost = if discount == 0 {
            self.minting_cost
        } else {
            require_gt!(
                constants::MARKET_USD_UNIT,
                discount,
                CoreError::InvalidGTDiscount
            );
            let discounted_factor = constants::MARKET_USD_UNIT - discount;

            apply_factor::<_, { constants::MARKET_DECIMALS }>(
                &self.minting_cost,
                &discounted_factor,
            )
            .ok_or(error!(CoreError::InvalidGTDiscount))?
        };

        require!(minting_cost != 0, CoreError::InvalidGTConfig);

        let remainder = size_in_value % minting_cost;
        let minted = (size_in_value / minting_cost)
            .try_into()
            .map_err(|_| error!(CoreError::TokenAmountOverflow))?;

        let minted_value = size_in_value - remainder;

        msg!(
            "[GT] will mint {} units of GT with a minting cost of {} per unit GT (in terms of trade volume), discount = {}",
            minted,
            minting_cost,
            discount,
        );

        Ok((minted, minted_value))
    }

    pub(crate) fn ranks(&self) -> &[u64] {
        &self.ranks[0..(self.max_rank as usize)]
    }

    /// Request an exchange.
    ///
    /// # CHECK
    /// - `user`, vault` and `exchange` must owned by this store.
    ///
    /// # Errors
    /// - `user`, `vault` and `exchange` must have been initialized.
    /// - `vault` must be depositable.
    /// - `user` must have enough amount of GT.
    pub(crate) fn unchecked_request_exchange(
        &mut self,
        user: &mut UserHeader,
        vault: &mut GtExchangeVault,
        exchange: &mut GtExchange,
        amount: u64,
    ) -> Result<()> {
        require!(user.is_initialized(), CoreError::InvalidArgument);
        require!(vault.is_initialized(), CoreError::InvalidArgument);
        require!(exchange.is_initialized(), CoreError::InvalidArgument);

        self.unchecked_burn_from(user, amount)?;

        vault.add(amount)?;
        exchange.add(amount)?;

        Ok(())
    }

    /// Confirm the exchange vault.
    ///
    /// # CHECK
    /// - `vault` must be owned by this store.
    ///
    /// # Errors
    /// - `vault` must have been initialized.
    /// - `vault` must be confirmable.
    pub(crate) fn unchecked_confirm_exchange_vault(
        &mut self,
        vault: &mut GtExchangeVault,
    ) -> Result<()> {
        require!(vault.is_initialized(), CoreError::InvalidArgument);

        let amount = vault.confirm()?;

        self.process_es_gt(amount)?;

        Ok(())
    }

    fn process_es_gt(&mut self, amount: u64) -> Result<()> {
        use gmsol_model::utils::{apply_factor, div_to_factor};

        if amount == 0 || self.supply == 0 {
            return Ok(());
        }

        let mut amount = u128::from(amount);
        let amount_for_vault =
            apply_factor::<_, { constants::MARKET_DECIMALS }>(&amount, &self.es_receiver_factor())
                .ok_or(error!(CoreError::ValueOverflow))?;
        require_gte!(amount, amount_for_vault, CoreError::Internal);
        amount = amount
            .checked_sub(amount_for_vault)
            .ok_or(error!(CoreError::Internal))?;

        let next_es_vault = self
            .es_vault
            .checked_add(amount_for_vault.try_into()?)
            .ok_or(error!(CoreError::TokenAmountOverflow))?;

        debug_assert_ne!(self.supply, 0);
        let supply = u128::from(self.supply);

        let delta = div_to_factor::<_, { constants::MARKET_DECIMALS }>(&amount, &supply, false)
            .ok_or(error!(CoreError::ValueOverflow))?;

        let next_es_factor = self
            .es_factor
            .checked_add(delta)
            .ok_or(error!(CoreError::ValueOverflow))?;

        self.es_vault = next_es_vault;
        self.es_factor = next_es_factor;
        Ok(())
    }
}

type GtExchangeVaultFlagsMap = bitmaps::Bitmap<MAX_FLAGS>;
type GtExchangeVaultFlagsValue = u8;

/// GT Exchange Vault Flags.
#[repr(u8)]
#[non_exhaustive]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum GtExchangeVaultFlag {
    /// Initialized.
    Intiailized,
    /// Confirmed.
    Comfirmed,
    // CHECK: should have no more than `MAX_FLAGS` of flags.
}

/// GT Exchange Vault.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct GtExchangeVault {
    bump: u8,
    flags: GtExchangeVaultFlagsValue,
    padding: [u8; 6],
    ts: i64,
    time_window: i64,
    amount: u64,
    store: Pubkey,
    reserved: [u8; 64],
}

impl GtExchangeVault {
    fn get_flag(&self, kind: GtExchangeVaultFlag) -> bool {
        let index = u8::from(kind);
        let map = GtExchangeVaultFlagsMap::from_value(self.flags);
        map.get(usize::from(index))
    }

    fn set_flag(&mut self, kind: GtExchangeVaultFlag, value: bool) -> bool {
        let index = u8::from(kind);
        let mut map = GtExchangeVaultFlagsMap::from_value(self.flags);
        map.set(usize::from(index), value)
    }

    /// Get amount.
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get whether the vault is initialized.
    pub fn is_initialized(&self) -> bool {
        self.get_flag(GtExchangeVaultFlag::Intiailized)
    }

    /// Get whether the vault is comfirmed.
    pub fn is_confirmed(&self) -> bool {
        self.get_flag(GtExchangeVaultFlag::Comfirmed)
    }

    pub(crate) fn init(&mut self, store: &Pubkey, time_window: u32) -> Result<()> {
        require!(!self.is_initialized(), CoreError::PreconditionsAreNotMet);

        require!(time_window != 0, CoreError::InvalidArgument);

        let clock = Clock::get()?;

        self.ts = clock.unix_timestamp;
        self.store = *store;
        self.set_flag(GtExchangeVaultFlag::Intiailized, true);
        self.time_window = i64::from(time_window);

        Ok(())
    }

    /// Get current time window index.
    pub fn time_window_index(&self) -> i64 {
        get_time_window_index(self.ts, self.time_window)
    }

    fn validate_confirmable(&self) -> Result<()> {
        require!(self.is_initialized(), CoreError::PreconditionsAreNotMet);
        require!(!self.is_confirmed(), CoreError::PreconditionsAreNotMet);

        let clock = Clock::get()?;
        let current_index = get_time_window_index(clock.unix_timestamp, self.time_window);

        require_gt!(
            current_index,
            self.time_window_index(),
            CoreError::PreconditionsAreNotMet
        );

        Ok(())
    }

    /// Confirm the vault.
    fn confirm(&mut self) -> Result<u64> {
        self.validate_confirmable()?;
        self.set_flag(GtExchangeVaultFlag::Comfirmed, true);
        Ok(self.amount)
    }

    fn validate_depositable(&self) -> Result<()> {
        require!(!self.is_confirmed(), CoreError::PreconditionsAreNotMet);

        let clock = Clock::get()?;
        let current_index = get_time_window_index(clock.unix_timestamp, self.time_window);
        require_eq!(
            current_index,
            self.time_window_index(),
            CoreError::InvalidArgument
        );
        Ok(())
    }

    /// Add GT to this vault.
    ///
    /// # Errors
    /// - This vault must be depositable.
    /// - Error on amount overflow.
    fn add(&mut self, amount: u64) -> Result<()> {
        self.validate_depositable()?;

        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(error!(CoreError::TokenAmountOverflow))?;
        Ok(())
    }
}

impl Seed for GtExchangeVault {
    const SEED: &'static [u8] = b"gt_exchange_vault";
}

impl gmsol_utils::InitSpace for GtExchangeVault {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

type GtExchangeFlagsMap = bitmaps::Bitmap<MAX_FLAGS>;
type GtExchangeFlagsValue = u8;

/// GT Exchange Vault Flags.
#[repr(u8)]
#[non_exhaustive]
#[derive(num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
pub enum GtExchangeFlag {
    /// Initialized.
    Intiailized,
    // CHECK: should have no more than `MAX_FLAGS` of flags.
}

/// GT Exchange Account.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct GtExchange {
    bump: u8,
    flags: GtExchangeFlagsValue,
    padding: [u8; 6],
    amount: u64,
    owner: Pubkey,
    store: Pubkey,
    vault: Pubkey,
    reserved: [u8; 64],
}

impl GtExchange {
    fn get_flag(&self, kind: GtExchangeFlag) -> bool {
        let index = u8::from(kind);
        let map = GtExchangeFlagsMap::from_value(self.flags);
        map.get(usize::from(index))
    }

    fn set_flag(&mut self, kind: GtExchangeFlag, value: bool) -> bool {
        let index = u8::from(kind);
        let mut map = GtExchangeFlagsMap::from_value(self.flags);
        map.set(usize::from(index), value)
    }

    /// Get whether the vault is initialized.
    pub fn is_initialized(&self) -> bool {
        self.get_flag(GtExchangeFlag::Intiailized)
    }

    pub(crate) fn init(&mut self, owner: &Pubkey, store: &Pubkey, vault: &Pubkey) -> Result<()> {
        require!(self.is_initialized(), CoreError::PreconditionsAreNotMet);

        self.owner = *owner;
        self.store = *store;
        self.vault = *vault;

        Ok(())
    }

    /// Add GT amount.
    fn add(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(error!(CoreError::TokenAmountOverflow))?;
        Ok(())
    }
}

fn get_time_window_index(ts: i64, time_window: i64) -> i64 {
    debug_assert!(time_window > 0);
    ts / time_window
}
