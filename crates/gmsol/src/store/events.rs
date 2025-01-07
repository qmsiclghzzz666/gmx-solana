use crate::{
    decode::untagged,
    types::{
        self, DepositExecuted, DepositRemoved, MarketStateUpdated, OrderRemoved,
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
        OrderRemoved,
        TradeEvent,
        MarketStateUpdated
    ]
);
