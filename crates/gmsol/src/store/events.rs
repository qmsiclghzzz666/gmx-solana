use crate::{
    decode::untagged,
    types::{RemoveDepositEvent, RemoveOrderEvent, RemoveWithdrawalEvent, TradeEvent},
};

untagged!(
    StoreCPIEvent,
    [
        RemoveDepositEvent,
        RemoveOrderEvent,
        RemoveWithdrawalEvent,
        TradeEvent
    ]
);
