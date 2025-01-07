use crate::{
    decode::untagged,
    types::{self, DepositExecuted, DepositRemoved, OrderRemoved, WithdrawalRemoved},
};

type TradeEvent = types::Trade<'static>;

untagged!(
    StoreCPIEvent,
    [
        DepositRemoved,
        DepositExecuted,
        OrderRemoved,
        WithdrawalRemoved,
        TradeEvent
    ]
);
