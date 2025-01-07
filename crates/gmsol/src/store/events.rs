use crate::{
    decode::untagged,
    types::{
        self, DepositExecuted, DepositRemoved, GlvDepositRemoved, GlvWithdrawalRemoved,
        MarketStateUpdated, OrderRemoved, PositionIncreased, ShiftRemoved, SwapExecuted,
        WithdrawalExecuted, WithdrawalRemoved,
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
        OrderRemoved,
        TradeEvent,
        MarketStateUpdated,
        SwapExecuted
    ]
);
