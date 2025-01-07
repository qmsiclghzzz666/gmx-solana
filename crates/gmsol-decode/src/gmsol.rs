use crate::{impl_decode_for_cpi_event, impl_decode_for_zero_copy};

use gmsol_store::{
    events::{
        DepositExecuted, DepositRemoved, GlvDepositRemoved, GlvWithdrawalRemoved,
        MarketStateUpdated, OrderRemoved, PositionIncreased, ShiftRemoved, SwapExecuted,
        Trade as TradeEvent, WithdrawalExecuted, WithdrawalRemoved,
    },
    states::{Deposit, Market, Order, Position, Store, Withdrawal},
};

use crate::{untagged, value::UnknownOwnedData};

impl_decode_for_zero_copy!(Store);
impl_decode_for_zero_copy!(Position);
impl_decode_for_zero_copy!(Market);
impl_decode_for_zero_copy!(Deposit);
impl_decode_for_zero_copy!(Withdrawal);
impl_decode_for_zero_copy!(Order);

impl_decode_for_cpi_event!(DepositRemoved);
impl_decode_for_cpi_event!(DepositExecuted);
impl_decode_for_cpi_event!(WithdrawalRemoved);
impl_decode_for_cpi_event!(WithdrawalExecuted);
impl_decode_for_cpi_event!(ShiftRemoved);
impl_decode_for_cpi_event!(GlvDepositRemoved);
impl_decode_for_cpi_event!(GlvWithdrawalRemoved);
impl_decode_for_cpi_event!(PositionIncreased);
impl_decode_for_cpi_event!(OrderRemoved);
impl_decode_for_cpi_event!(TradeEvent<'static>);
impl_decode_for_cpi_event!(MarketStateUpdated);
impl_decode_for_cpi_event!(SwapExecuted);

untagged!(
    GMSOLAccountData,
    [
        Deposit,
        Withdrawal,
        Order,
        Store,
        Market,
        Position,
        UnknownOwnedData
    ]
);

type Account = crate::value::Account<GMSOLAccountData>;

type Trade = TradeEvent<'static>;

untagged!(
    GMSOLCPIEvent,
    [
        DepositExecuted,
        DepositRemoved,
        WithdrawalExecuted,
        WithdrawalRemoved,
        ShiftRemoved,
        GlvDepositRemoved,
        GlvWithdrawalRemoved,
        PositionIncreased,
        OrderRemoved,
        Trade,
        MarketStateUpdated,
        SwapExecuted,
        UnknownOwnedData
    ]
);

type CPIEvents = crate::value::AnchorCPIEvents<GMSOLCPIEvent>;

untagged!(GMSOLData, [Account, CPIEvents]);
