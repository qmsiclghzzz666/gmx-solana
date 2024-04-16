use anchor_lang::prelude::*;

use crate::DataStoreError;

use super::{
    common::{SwapParams, TokensWithFeed},
    position::PositionKind,
    NonceBytes, Seed,
};

/// Order.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Order {
    /// The fixed-size part of the order.
    pub fixed: Box<Fixed>,
    /// The config for prices.
    pub prices: TokensWithFeed,
    /// The swap config.
    pub swap: SwapParams,
}

impl Order {
    /// Init space.
    pub fn init_space(tokens_with_feed: &[(Pubkey, Pubkey)], swap: &SwapParams) -> usize {
        Fixed::INIT_SPACE
            + TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap.long_token_swap_path.len(),
                swap.short_token_swap_path.len(),
            )
    }

    /// Initialize the order.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        bump: u8,
        nonce: &NonceBytes,
        market: &Pubkey,
        user: &Pubkey,
        position: Option<&Pubkey>,
        params: &OrderParams,
        tokens: &Tokens,
        senders: &Senders,
        receivers: &Receivers,
        tokens_with_feed: Vec<(Pubkey, Pubkey)>,
        swap: SwapParams,
    ) -> Result<()> {
        self.fixed.init(
            bump, nonce, market, user, position, params, tokens, senders, receivers,
        )?;
        self.prices = TokensWithFeed::from_vec(tokens_with_feed);
        self.swap = swap;
        Ok(())
    }
}

impl Seed for Order {
    const SEED: &'static [u8] = b"order";
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
    /// The order market.
    pub market: Pubkey,
    /// The creator of the order.
    pub user: Pubkey,
    /// Position.
    pub position: Option<Pubkey>,
    /// The params of order.
    pub params: OrderParams,
    /// The token config.
    pub tokens: Tokens,
    /// Senders.
    pub senders: Senders,
    /// Receivers.
    pub receivers: Receivers,
}

impl Fixed {
    #[allow(clippy::too_many_arguments)]
    fn init(
        &mut self,
        bump: u8,
        nonce: &NonceBytes,
        market: &Pubkey,
        user: &Pubkey,
        position: Option<&Pubkey>,
        params: &OrderParams,
        tokens: &Tokens,
        senders: &Senders,
        receivers: &Receivers,
    ) -> Result<()> {
        self.bump = bump;
        self.nonce = *nonce;
        self.updated_at_slot = Clock::get()?.slot;
        self.market = *market;
        self.user = *user;
        self.position = position.copied();
        self.params = params.clone();
        self.tokens = tokens.clone();
        self.senders = senders.clone();
        self.receivers = receivers.clone();
        Ok(())
    }
}

/// Senders.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Senders {
    /// The token account for sending inital collateral tokens.
    pub initial_collateral_token_account: Option<Pubkey>,
}

/// Fees and tokens receivers for [`Order`]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Receivers {
    /// The ui fee receiver.
    pub ui_fee: Pubkey,
    /// The token account for receiving the final output tokens.
    pub final_output_token_account: Option<Pubkey>,
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
    /// The initial collateral token or swap in token.
    pub initial_collateral_token: Pubkey,
    /// The expected collateral token or swap out token.
    pub output_token: Pubkey,
    /// The expected pnl token.
    pub secondary_output_token: Pubkey,
    /// Final output token.
    pub final_output_token: Option<Pubkey>,
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
    pub acceptable_price: Option<u128>,
    /// Whether the order is for a long or short position.
    pub is_long: bool,
}

impl OrderParams {
    /// Get position kind.
    pub fn to_position_kind(&self) -> Result<PositionKind> {
        match &self.kind {
            OrderKind::MarketSwap => Err(DataStoreError::PositionIsNotRequried.into()),
            OrderKind::Liquidation | OrderKind::MarketDecrease | OrderKind::MarketIncrease => {
                if self.is_long {
                    Ok(PositionKind::Long)
                } else {
                    Ok(PositionKind::Short)
                }
            }
        }
    }
}

/// Order Kind.
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
