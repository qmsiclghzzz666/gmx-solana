use std::{borrow::Cow, ops::Deref};

use anchor_lang::prelude::*;
use bitmaps::Bitmap;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::Zeroable;
use gmsol_model::{
    action::{
        decrease_position::DecreasePositionReport, increase_position::IncreasePositionReport,
    },
    params::fee::PositionFees,
    price::Prices,
};

use crate::{
    constants,
    states::{
        common::action::ActionState,
        order::{OrderKind, TransferOut},
        position::PositionState,
        Position,
    },
    CoreError,
};

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct DepositCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Deposit account.
    pub deposit: Pubkey,
}

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WithdrawalCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Withdrawal account.
    pub withdrawal: Pubkey,
}

#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OrderCreatedEvent {
    /// Event time.
    pub ts: i64,
    /// Store account.
    pub store: Pubkey,
    /// Order account.
    pub order: Pubkey,
    /// Position account.
    pub position: Option<Pubkey>,
}

/// Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
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
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    pub reason: String,
}

impl RemoveDepositEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        deposit: Pubkey,
        market_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
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
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// Order removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
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
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
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
        owner: Pubkey,
        state: ActionState,
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
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
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
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    pub reason: String,
}

impl RemoveWithdrawalEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        withdrawal: Pubkey,
        market_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
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
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// Shift removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct RemoveShiftEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// Shift.
    pub shift: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    pub reason: String,
}

impl RemoveShiftEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        shift: Pubkey,
        market_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            shift,
            market_token,
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// GLV Deposit removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct RemoveGlvDepositEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// GLV Deposit.
    pub glv_deposit: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// GLV token.
    pub glv_token: Pubkey,
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    pub reason: String,
}

impl RemoveGlvDepositEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        glv_deposit: Pubkey,
        market_token: Pubkey,
        glv_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            glv_deposit,
            glv_token,
            market_token,
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// GLV Withdrawal removed event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct RemoveGlvWithdrawalEvent {
    /// Action id.
    pub id: u64,
    /// Timestamp.
    pub ts: i64,
    /// Slot.
    pub slot: u64,
    /// Store.
    pub store: Pubkey,
    /// GLV Withdrawal
    pub glv_withdrawal: Pubkey,
    /// Market token.
    pub market_token: Pubkey,
    /// GLV token.
    pub glv_token: Pubkey,
    /// Owner.
    pub owner: Pubkey,
    /// Final state.
    pub state: ActionState,
    /// Reason.
    pub reason: String,
}

impl RemoveGlvWithdrawalEvent {
    pub(crate) fn new(
        id: u64,
        store: Pubkey,
        glv_withdrawal: Pubkey,
        market_token: Pubkey,
        glv_token: Pubkey,
        owner: Pubkey,
        state: ActionState,
        reason: impl ToString,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            id,
            ts: clock.unix_timestamp,
            slot: clock.slot,
            store,
            glv_withdrawal,
            glv_token,
            market_token,
            owner,
            state,
            reason: reason.to_string(),
        })
    }
}

/// Trade event.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct TradeEvent<'a>(Cow<'a, TradeEventData>);

impl<'a> From<&'a TradeEventData> for TradeEvent<'a> {
    fn from(value: &'a TradeEventData) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl<'a> anchor_lang::Event for TradeEvent<'a> {
    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&[189, 219, 127, 211, 78, 230, 97, 238]);
        self.serialize(&mut data).unwrap();
        data
    }
}

impl<'a> anchor_lang::Discriminator for TradeEvent<'a> {
    const DISCRIMINATOR: [u8; 8] = [189, 219, 127, 211, 78, 230, 97, 238];
}

impl<'a> TradeEvent<'a> {
    /// Emit this event through CPI. This is a manual implementation of `emit_cpi!`.
    pub(crate) fn emit(&self, event_authority: &AccountInfo, bump: u8) -> Result<()> {
        use anchor_lang::{solana_program::instruction::Instruction, Discriminator};

        let authority_info = event_authority.to_account_info();
        let authority_bump = bump;
        let disc = anchor_lang::event::EVENT_IX_TAG_LE;
        let mut ix_data = Vec::with_capacity(16 + TradeEventData::INIT_SPACE);
        ix_data.extend_from_slice(&disc);
        ix_data.extend_from_slice(&Self::DISCRIMINATOR);
        self.serialize(&mut ix_data)?;
        let ix = Instruction {
            program_id: crate::ID,
            accounts: vec![AccountMeta::new_readonly(*authority_info.key, true)],
            data: ix_data,
        };
        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[authority_info],
            &[&[constants::EVENT_AUTHORITY_SEED, &[authority_bump]]],
        )?;
        Ok(())
    }
}

#[cfg(feature = "display")]
impl<'a> std::fmt::Display for TradeEvent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradeEvent")
            .field("trade_id", &self.trade_id)
            .field("store", &self.store.to_string())
            .field("market_token", &self.market_token.to_string())
            .field("user", &self.user.to_string())
            .field("position", &self.position.to_string())
            .field("order", &self.order.to_string())
            .field(
                "final_output_token",
                &(self.final_output_token == Pubkey::default()).then_some(self.final_output_token),
            )
            .field("ts", &self.ts)
            .field("slot", &self.slot)
            .field("is_long", &self.is_long())
            .field("is_collateral_long", &self.is_collateral_long())
            .field("is_increase", &self.is_increase())
            .field("delta_collateral_amount", &self.delta_collateral_amount())
            .field("delta_size_in_usd", &self.delta_size_in_usd())
            .field("delta_size_in_tokens", &self.delta_size_in_tokens())
            .field("prices", &self.prices)
            .field("execution_price", &self.execution_price)
            .field("price_impact_value", &self.price_impact_value)
            .field("price_impact_diff", &self.price_impact_diff)
            .field("pnl", &self.pnl)
            .field("fees", &self.fees)
            .field("output_amounts", &self.output_amounts)
            .field("transfer_out", &self.transfer_out)
            .finish_non_exhaustive()
    }
}

/// Trade event data.
#[account(zero_copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TradeEventData {
    /// Trade flag.
    flags: u8,
    padding_0: [u8; 7],
    /// Trade id.
    pub trade_id: u64,
    /// Authority.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub authority: Pubkey,
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
    /// Order address.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub order: Pubkey,
    /// Final output token.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_with::As::<serde_with::DisplayFromStr>")
    )]
    pub final_output_token: Pubkey,
    /// Trade ts.
    pub ts: i64,
    /// Trade slot.
    pub slot: u64,
    /// Before state.
    pub before: PositionState,
    /// After state.
    pub after: PositionState,
    /// Transfer out.
    pub transfer_out: TransferOut,
    padding_1: [u8; 8],
    /// Prices.
    pub prices: TradePrices,
    /// Execution price.
    pub execution_price: u128,
    /// Price impact value.
    pub price_impact_value: i128,
    /// Price impact diff.
    pub price_impact_diff: u128,
    /// Processed pnl.
    pub pnl: TradePnL,
    /// Fees.
    pub fees: TradeFees,
    /// Output amounts.
    #[cfg_attr(feature = "serde", serde(default))]
    pub output_amounts: TradeOutputAmounts,
}

impl TradeEventData {
    /// Init space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Seed.
    pub const SEED: &'static [u8] = b"trade_event_data";
}

/// Price.
#[zero_copy]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TradePrice {
    /// Min price.
    pub min: u128,
    /// Max price.
    pub max: u128,
}

/// Prices.
#[zero_copy]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TradePrices {
    /// Index token price.
    pub index: TradePrice,
    /// Long token price.
    pub long: TradePrice,
    /// Short token price.
    pub short: TradePrice,
}

impl TradePrices {
    fn set_with_prices(&mut self, prices: &Prices<u128>) {
        self.index.min = prices.index_token_price.min;
        self.index.max = prices.index_token_price.max;
        self.long.min = prices.long_token_price.min;
        self.long.max = prices.long_token_price.max;
        self.short.min = prices.short_token_price.min;
        self.short.max = prices.short_token_price.max;
    }
}

/// Trade PnL.
#[zero_copy]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TradePnL {
    /// Final PnL value.
    pub pnl: i128,
    /// Uncapped PnL value.
    pub uncapped_pnl: i128,
}

/// Trade Fees.
#[zero_copy]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TradeFees {
    /// Order fee for receiver amount.
    pub order_fee_for_receiver_amount: u128,
    /// Order fee for pool amount.
    pub order_fee_for_pool_amount: u128,
    /// Total borrowing fee amount.
    pub total_borrowing_fee_amount: u128,
    /// Borrowing fee for receiver amount.
    pub borrowing_fee_for_receiver_amount: u128,
    /// Funding fee amount.
    pub funding_fee_amount: u128,
    /// Claimable funding fee long token amount.
    pub claimable_funding_fee_long_token_amount: u128,
    /// Claimable funding fee short token amount.
    pub claimable_funding_fee_short_token_amount: u128,
}

impl TradeFees {
    fn set_with_position_fees(&mut self, fees: &PositionFees<u128>) {
        self.order_fee_for_receiver_amount = *fees.order_fee().fee_receiver_amount();
        self.order_fee_for_pool_amount = *fees.order_fee().fee_amount_for_pool();
        self.total_borrowing_fee_amount = *fees.borrowing().amount();
        self.borrowing_fee_for_receiver_amount = *fees.borrowing().amount_for_receiver();
        self.funding_fee_amount = *fees.funding_fees().amount();
        self.claimable_funding_fee_long_token_amount =
            *fees.funding_fees().claimable_long_token_amount();
        self.claimable_funding_fee_short_token_amount =
            *fees.funding_fees().claimable_short_token_amount();
    }
}

/// Output amounts.
#[zero_copy]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct TradeOutputAmounts {
    /// Output amount.
    pub output_amount: u128,
    /// Secondary output amount.
    pub secondary_output_amount: u128,
}

type TradeFlagMap = Bitmap<8>;

#[allow(clippy::enum_variant_names)]
#[repr(u8)]
enum TradeFlag {
    /// Is long.
    IsLong,
    /// Is collateral long.
    IsCollateralLong,
    /// Is increase.
    IsIncrease,
}

impl TradeEventData {
    pub(crate) fn init(
        &mut self,
        is_increase: bool,
        is_collateral_long: bool,
        pubkey: Pubkey,
        position: &Position,
        order: Pubkey,
    ) -> Result<&mut Self> {
        let clock = Clock::get()?;
        self.set_flags(position.try_is_long()?, is_collateral_long, is_increase);
        self.trade_id = 0;
        require_eq!(self.store, position.store, CoreError::PermissionDenied);
        self.market_token = position.market_token;
        self.user = position.owner;
        self.position = pubkey;
        self.order = order;
        self.final_output_token = Pubkey::default();
        self.ts = clock.unix_timestamp;
        self.slot = clock.slot;
        self.before = position.state;
        self.after = position.state;
        self.transfer_out = TransferOut::zeroed();
        self.prices = TradePrices::zeroed();
        self.execution_price = 0;
        self.price_impact_value = 0;
        self.price_impact_diff = 0;
        self.pnl = TradePnL::zeroed();
        self.fees = TradeFees::zeroed();
        self.output_amounts = TradeOutputAmounts::zeroed();
        Ok(self)
    }

    fn set_flags(
        &mut self,
        is_long: bool,
        is_collateral_long: bool,
        is_increase: bool,
    ) -> &mut Self {
        let mut flags = TradeFlagMap::new();
        flags.set(TradeFlag::IsLong as usize, is_long);
        flags.set(TradeFlag::IsCollateralLong as usize, is_collateral_long);
        flags.set(TradeFlag::IsIncrease as usize, is_increase);
        self.flags = flags.into_value();
        self
    }

    fn get_flag(&self, flag: TradeFlag) -> bool {
        let map = TradeFlagMap::from_value(self.flags);
        map.get(flag as usize)
    }

    /// Return whether the position side is long.
    pub fn is_long(&self) -> bool {
        self.get_flag(TradeFlag::IsLong)
    }

    /// Return whether the collateral side is long.
    pub fn is_collateral_long(&self) -> bool {
        self.get_flag(TradeFlag::IsCollateralLong)
    }

    /// Return whether the trade is caused by an increase order.
    pub fn is_increase(&self) -> bool {
        self.get_flag(TradeFlag::IsIncrease)
    }

    fn validate(&self) -> Result<()> {
        require_gt!(
            self.trade_id,
            self.before.trade_id,
            CoreError::InvalidTradeID
        );
        if self.is_increase() {
            require_gte!(
                self.after.size_in_usd,
                self.before.size_in_usd,
                CoreError::InvalidTradeDeltaSize
            );
            require_gte!(
                self.after.size_in_tokens,
                self.before.size_in_tokens,
                CoreError::InvalidTradeDeltaTokens
            );
        } else {
            require_gte!(
                self.before.size_in_usd,
                self.after.size_in_usd,
                CoreError::InvalidTradeDeltaSize
            );
            require_gte!(
                self.before.size_in_tokens,
                self.after.size_in_tokens,
                CoreError::InvalidTradeDeltaTokens
            );
        }
        require_gte!(
            self.after.borrowing_factor,
            self.before.borrowing_factor,
            CoreError::InvalidBorrowingFactor
        );
        require_gte!(
            self.after.funding_fee_amount_per_size,
            self.before.funding_fee_amount_per_size,
            CoreError::InvalidFundingFactors
        );
        require_gte!(
            self.after.long_token_claimable_funding_amount_per_size,
            self.before.long_token_claimable_funding_amount_per_size,
            CoreError::InvalidFundingFactors
        );
        require_gte!(
            self.after.short_token_claimable_funding_amount_per_size,
            self.before.short_token_claimable_funding_amount_per_size,
            CoreError::InvalidFundingFactors
        );
        Ok(())
    }

    /// Update with new position state.
    pub(crate) fn update_with_state(&mut self, new_state: &PositionState) -> Result<()> {
        self.trade_id = new_state.trade_id;
        self.after = *new_state;
        self.validate()?;
        Ok(())
    }

    /// Update with transfer out.
    #[inline(never)]
    pub(crate) fn update_with_transfer_out(&mut self, transfer_out: &TransferOut) -> Result<()> {
        self.transfer_out = *transfer_out;
        self.transfer_out.set_executed(true);
        Ok(())
    }

    pub(crate) fn set_final_output_token(&mut self, token: &Pubkey) {
        self.final_output_token = *token;
    }

    /// Update with increase report.
    #[inline(never)]
    pub(crate) fn update_with_increase_report(
        &mut self,
        report: &IncreasePositionReport<u128>,
    ) -> Result<()> {
        self.prices.set_with_prices(report.params().prices());
        self.execution_price = *report.execution().execution_price();
        self.price_impact_value = *report.execution().price_impact_value();
        self.fees.set_with_position_fees(report.fees());
        Ok(())
    }

    /// Update with decrease report.
    pub(crate) fn update_with_decrease_report(
        &mut self,
        report: &DecreasePositionReport<u128>,
        prices: &Prices<u128>,
    ) -> Result<()> {
        self.prices.set_with_prices(prices);
        self.execution_price = *report.execution_price();
        self.price_impact_value = *report.price_impact_value();
        self.price_impact_diff = *report.price_impact_diff();
        self.pnl.pnl = *report.pnl().pnl();
        self.pnl.uncapped_pnl = *report.pnl().uncapped_pnl();
        self.fees.set_with_position_fees(report.fees());
        self.output_amounts.output_amount = *report.output_amounts().output_amount();
        self.output_amounts.secondary_output_amount =
            *report.output_amounts().secondary_output_amount();
        Ok(())
    }
}

impl<'a> Deref for TradeEvent<'a> {
    type Target = TradeEventData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "utils")]
impl<'a> TradeEvent<'a> {
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

    #[cfg(feature = "utils")]
    /// Create position from this event.
    pub fn to_position(&self, meta: &impl crate::states::HasMarketMeta) -> Position {
        use crate::states::position::PositionKind;

        let mut position = Position::default();

        let kind = if self.is_long() {
            PositionKind::Long
        } else {
            PositionKind::Short
        };

        let collateral_token = if self.is_collateral_long() {
            meta.market_meta().long_token_mint
        } else {
            meta.market_meta().short_token_mint
        };

        // FIXME: should we provide a correct bump here?
        position
            .try_init(
                kind,
                0,
                self.store,
                &self.user,
                &self.market_token,
                &collateral_token,
            )
            .unwrap();
        position.state = self.after;
        position
    }
}
