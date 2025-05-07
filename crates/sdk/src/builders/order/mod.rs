/// Builder for the `create_order` instruction.
pub mod create;

/// Builder for the `close_order` instruction.
pub mod close;

/// Builder for the `update_order` instruction.
pub mod update;

/// Min execution lamports for order.
pub const MIN_EXECUTION_LAMPORTS_FOR_ORDER: u64 = 300_000;

pub use self::{
    close::{CloseOrder, CloseOrderHint},
    create::{
        CreateOrder, CreateOrderHint, CreateOrderKind, CreateOrderParams, DecreasePositionSwapType,
    },
    update::{UpdateOrder, UpdateOrderHint, UpdateOrderParams},
};
