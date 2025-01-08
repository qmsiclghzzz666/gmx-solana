use crate::{
    decode::untagged,
    types::{
        DepositExecuted, DepositRemoved, GlvDepositRemoved, GlvWithdrawalRemoved, GtUpdated,
        MarketFeesUpdated, MarketStateUpdated, OrderRemoved, PositionDecreased, PositionIncreased,
        ShiftRemoved, SwapExecuted, TradeEvent, WithdrawalExecuted, WithdrawalRemoved,
    },
};

untagged!(
    StoreCPIEvent,
    [
        DepositExecuted,
        DepositRemoved,
        WithdrawalExecuted,
        WithdrawalRemoved,
        ShiftRemoved,
        GlvDepositRemoved,
        GlvWithdrawalRemoved,
        PositionIncreased,
        PositionDecreased,
        OrderRemoved,
        TradeEvent,
        MarketFeesUpdated,
        MarketStateUpdated,
        SwapExecuted,
        GtUpdated
    ]
);
