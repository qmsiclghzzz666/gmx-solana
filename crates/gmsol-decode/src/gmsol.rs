use crate::{
    impl_decode_for_account_deserialize, impl_decode_for_cpi_event, impl_decode_for_zero_copy,
};

use gmsol_store::{
    events::{RemoveDepositEvent, RemoveOrderEvent, RemoveWithdrawalEvent, TradeEvent},
    states::{Deposit, Market, Order, Position, Store, Withdrawal},
};

use crate::{untagged, value::UnknownOwnedData};

impl_decode_for_zero_copy!(Store);
impl_decode_for_zero_copy!(Position);
impl_decode_for_zero_copy!(Market);
impl_decode_for_account_deserialize!(Deposit);
impl_decode_for_account_deserialize!(Withdrawal);
impl_decode_for_account_deserialize!(Order);
impl_decode_for_cpi_event!(RemoveDepositEvent);
impl_decode_for_cpi_event!(RemoveWithdrawalEvent);
impl_decode_for_cpi_event!(RemoveOrderEvent);
impl_decode_for_cpi_event!(TradeEvent);

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

untagged!(
    GMSOLCPIEvent,
    [
        RemoveDepositEvent,
        RemoveOrderEvent,
        RemoveWithdrawalEvent,
        TradeEvent,
        UnknownOwnedData
    ]
);

type CPIEvents = crate::value::AnchorCPIEvents<GMSOLCPIEvent>;

untagged!(GMSOLData, [Account, CPIEvents]);
