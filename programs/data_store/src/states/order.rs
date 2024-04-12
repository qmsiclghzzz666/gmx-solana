use anchor_lang::prelude::*;

/// Order.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Order {
    /// The fixed-size part of the order.
    pub fixed: Fixed,
}

/// Fixed part of [`Order`]
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Fixed {
    /// The bump seed.
    pub bump: u8,
    /// The nonce bytes for this order.
    pub nonce: [u8; 32],
    /// The slot that the order was last updated at.
    pub updated_at_slot: u64,
    /// The creator of the order.
    pub user: Pubkey,
    /// The params of order.
    pub params: OrderParams,
    /// The token config.
    pub tokens: Tokens,
    /// Senders.
    pub senders: Senders,
    /// Receivers.
    pub receivers: Receivers,
}

/// Senders.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Senders {
    /// The token account for sending inital collateral tokens.
    pub initial_collateral_token_account: Pubkey,
}

/// Fees and tokens receivers for [`Order`]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Receivers {
    /// The ui fee receiver.
    pub ui_fee: Pubkey,
    /// The token account for receiving the output tokens.
    pub output_token_account: Pubkey,
    /// The token account for receiving the secondary output tokens.
    pub secondary_output_token_account: Option<Pubkey>,
}

/// Token config for [`Order`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tokens {
    /// The market token mint of the market order.
    ///
    /// Used to identify the market.
    pub market_token: Pubkey,
    /// The initial collateral token.
    pub initial_collateral_token: Pubkey,
    /// The output token.
    pub output_token: Pubkey,
    /// The secondary output token.
    pub secondary_output_token: Option<Pubkey>,
}

/// The parameters for [`Order`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OrderParams {
    /// Order kind.
    pub kind: OrderKind,
    /// Min amount for output tokens.
    pub min_output_amount: u64,
    /// Size delta usd.
    pub size_delta_usd: u128,
    /// Initial collateral delta amount.
    pub initial_collateral_delta_amount: u64,
    /// Acceptable price (unit price).
    pub acceptable_price: u128,
    /// Whether the order is for a long or short position.
    pub is_long: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
pub enum OrderKind {
    /// Swap token A to token B at the current market price.
    ///
    /// The order will be cancelled if the `min_output_amount` cannot be fulfilled.
    MarketSwap,
    /// Increase position at the current market price.
    ///
    /// The order will be cancelled if the position cannot be increased at the acceptable price.
    MarketIncrease,
    /// Decrease position at the current market price.
    ///
    /// The order will be cancelled if the position cannot be decreased at the acceptable price.
    MarketDecrease,
    /// Liquidation: allows liquidation of positions if the criteria for liquidation are met.
    Liquidation,
}
