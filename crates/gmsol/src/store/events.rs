use crate::{
    decode::untagged,
    types::{
        self, DepositExecuted, DepositRemoved, GlvDepositRemoved, GlvWithdrawalRemoved,
        MarketStateUpdated, OrderRemoved, PositionDecreased, PositionIncreased, ShiftRemoved,
        SwapExecuted, WithdrawalExecuted, WithdrawalRemoved,
    },
};

type TradeEvent = types::Trade<'static>;

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
        MarketStateUpdated,
        SwapExecuted
    ]
);
