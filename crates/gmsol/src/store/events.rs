use crate::{
    decode::untagged,
    types::{self, DepositRemoved, OrderRemoved, WithdrawalRemoved},
};

type TradeEvent = types::Trade<'static>;

untagged!(
    StoreCPIEvent,
    [DepositRemoved, OrderRemoved, WithdrawalRemoved, TradeEvent]
);
