use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;
use gmsol_model::{
    action::{
        decrease_position::DecreasePositionReport, increase_position::IncreasePositionReport,
    },
    params::fee::PositionFees,
};

use crate::{
    states::{
        order::{OrderKind, TransferOut},
        position::PositionState,
        Position,
    },
    StoreError,
};

/// Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveDepositEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Deposit.
    pub deposit: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveDepositEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        deposit: Pubkey,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            deposit,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}

/// Order removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveOrderEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Order.
    pub order: Pubkey,
    /// Kind.
    pub kind: OrderKind,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveOrderEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        order: Pubkey,
        kind: OrderKind,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            kind,
            order,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}

/// Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RemoveWithdrawalEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Withdrawal.
    pub withdrawal: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// User.
    pub user: Pubkey,
    /// Reason.
    pub reason: String,
}

impl RemoveWithdrawalEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        withdrawal: Pubkey,
        market_token: Pubkey,
        user: Pubkey,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            withdrawal,
            market_token,
            user,
            reason: reason.to_string(),
        })
    }
}

/// Trade event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradeEvent(Box<TradeEventData>);

#[cfg(feature = "display")]
impl std::fmt::Display for TradeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradeEvent")
            .field("trade_id", &self.trade_id)
            .field("store", &self.store.to_string())
            .field("market_token", &self.market_token.to_string())
            .field("user", &self.user.to_string())
            .field("position", &self.position.to_string())
            .field("ts", &self.ts)
            .field("slot", &self.slot)
            .field("is_long", &self.is_long)
            .field("is_increase", &self.is_increase)
            .field("delta_collateral_amount", &self.delta_collateral_amount())
            .field("delta_size_in_usd", &self.delta_size_in_usd())
            .field("trade_cost", &self.trade_cost())
            .field("transfer_out", &self.transfer_out)
            .field("fees", &self.fees)
            .finish_non_exhaustive()
    }
}

/// Trade event data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct TradeEventData {
    /// Trade id.
    pub trade_id: u64,
    /// Store address.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub store: Pubkey,
    /// Market token.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub market_token: Pubkey,
    /// User.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub user: Pubkey,
    /// Position address.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub position: Pubkey,
    /// Trade ts.
    pub ts: i64,
    /// Trade slot.
    pub slot: u64,
    /// Trade side.
    pub is_long: bool,
    /// Trade direction.
    pub is_increase: bool,
    /// Before state.
    pub before: PositionState,
    /// After state.
    pub after: PositionState,
    /// Transfer out.
    pub transfer_out: TransferOut,
    /// Fees.
    pub fees: PositionFees<u128>,
    /// Reserved.
    #[cfg_attr(feature = "serde", serde(skip, default = "default_reserved"))]
    reserved: [u8; 128],
}

impl TradeEventData {
    fn validate(&self) -> Result<()> {
        require_gt!(
            self.trade_id,
            self.before.trade_id,
            StoreError::InvalidTradeID
        );
        if self.is_increase {
            require_gte!(
                self.after.size_in_usd,
                self.before.size_in_usd,
                StoreError::InvalidTradeDeltaSize
            );
            require_gte!(
                self.after.size_in_tokens,
                self.before.size_in_tokens,
                StoreError::InvalidTradeDeltaSize
            );
        } else {
            require_gte!(
                self.before.size_in_usd,
                self.after.size_in_usd,
                StoreError::InvalidTradeDeltaSize
            );
            require_gte!(
                self.before.size_in_tokens,
                self.after.size_in_tokens,
                StoreError::InvalidTradeDeltaSize
            );
        }
        require_gte!(
            self.after.borrowing_factor,
            self.before.borrowing_factor,
            StoreError::InvalidBorrowingFactor
        );
        require_gte!(
            self.after.funding_fee_amount_per_size,
            self.before.funding_fee_amount_per_size,
            StoreError::InvalidFundingFactors
        );
        require_gte!(
            self.after.long_token_claimable_funding_amount_per_size,
            self.before.long_token_claimable_funding_amount_per_size,
            StoreError::InvalidFundingFactors
        );
        require_gte!(
            self.after.short_token_claimable_funding_amount_per_size,
            self.before.short_token_claimable_funding_amount_per_size,
            StoreError::InvalidFundingFactors
        );
        Ok(())
    }
}

impl Deref for TradeEvent {
    type Target = TradeEventData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TradeEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn default_reserved() -> [u8; 128] {
    [0; 128]
}

impl TradeEvent {
    /// Create a new unchanged trade event.
    pub(crate) fn new_unchanged(
        is_increase: bool,
        pubkey: Pubkey,
        position: &Position,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self(Box::new(TradeEventData {
            trade_id: 0,
            store: position.store,
            market_token: position.market_token,
            user: position.owner,
            position: pubkey,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            is_long: position.is_long()?,
            is_increase,
            before: position.state,
            after: position.state,
            transfer_out: Default::default(),
            fees: Default::default(),
            reserved: default_reserved(),
        })))
    }

    /// Update with new position state.
    pub(crate) fn update_with_state(&mut self, new_state: &PositionState) -> Result<()> {
        self.trade_id = new_state.trade_id;
        self.after = *new_state;
        self.validate()?;
        Ok(())
    }

    /// Update with transfer out.
    pub(crate) fn update_with_transfer_out(&mut self, transfer_out: &TransferOut) -> Result<()> {
        self.transfer_out = transfer_out.clone();
        self.transfer_out.executed = true;
        Ok(())
    }

    /// Update with increase report.
    pub(crate) fn update_with_increase_report(
        &mut self,
        report: &IncreasePositionReport<u128>,
    ) -> Result<()> {
        self.fees = *report.fees();
        Ok(())
    }

    /// Update with decrease report.
    pub(crate) fn update_with_decrease_report(
        &mut self,
        report: &DecreasePositionReport<u128>,
    ) -> Result<()> {
        self.fees = *report.fees();
        Ok(())
    }
}

#[cfg(feature = "utils")]
impl TradeEvent {
    /// Updated at.
    pub fn updated_at(&self) -> i64 {
        self.after.increased_at.max(self.after.decreased_at)
    }

    /// Delta size in usd.
    pub fn delta_size_in_usd(&self) -> u128 {
        self.after.size_in_usd.abs_diff(self.before.size_in_usd)
    }

    /// Delta size in tokens.
    pub fn delta_size_in_tokens(&self) -> u128 {
        self.after
            .size_in_tokens
            .abs_diff(self.before.size_in_tokens)
    }

    /// Trade cost.
    pub fn trade_cost(&self) -> Option<u128> {
        self.delta_size_in_usd()
            .checked_div(self.delta_size_in_tokens())
    }

    /// Delta collateral amount.
    pub fn delta_collateral_amount(&self) -> u128 {
        self.after
            .collateral_amount
            .abs_diff(self.before.collateral_amount)
    }

    /// Delta borrowing factor.
    pub fn delta_borrowing_factor(&self) -> u128 {
        self.after
            .borrowing_factor
            .abs_diff(self.before.borrowing_factor)
    }

    /// Delta funding fee amount per size.
    pub fn delta_funding_fee_amount_per_size(&self) -> u128 {
        self.after
            .funding_fee_amount_per_size
            .abs_diff(self.before.funding_fee_amount_per_size)
    }

    /// Funding fee amount.
    pub fn funding_fee(&self) -> u128 {
        self.delta_funding_fee_amount_per_size()
            .saturating_mul(self.before.size_in_usd)
    }

    /// Delta claimable amount per size.
    pub fn delta_claimable_funding_amount_per_size(&self, is_long_token: bool) -> u128 {
        if is_long_token {
            self.after
                .long_token_claimable_funding_amount_per_size
                .abs_diff(self.before.long_token_claimable_funding_amount_per_size)
        } else {
            self.after
                .short_token_claimable_funding_amount_per_size
                .abs_diff(self.before.short_token_claimable_funding_amount_per_size)
        }
    }
}
