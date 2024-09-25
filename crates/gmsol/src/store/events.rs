use crate::{
    decode::untagged,
    types::{self, RemoveDepositEvent, RemoveOrderEvent, RemoveWithdrawalEvent},
};

type TradeEvent = types::TradeEvent<'static>;

untagged!(
    StoreCPIEvent,
    [
        RemoveDepositEvent,
        RemoveOrderEvent,
        RemoveWithdrawalEvent,
        TradeEvent
    ]
);
