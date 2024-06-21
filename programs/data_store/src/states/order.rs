use anchor_lang::prelude::*;
use gmx_core::action::decrease_position::DecreasePositionReport;

use crate::DataStoreError;

use super::{
    common::{SwapParams, TokenRecord, TokensWithFeed},
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
    pub fn init_space(tokens_with_feed: &[TokenRecord], swap: &SwapParams) -> usize {
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
        store: Pubkey,
        nonce: &NonceBytes,
        market: &Pubkey,
        user: &Pubkey,
        position: Option<&Pubkey>,
        params: &OrderParams,
        tokens: &Tokens,
        senders: &Senders,
        receivers: &Receivers,
        tokens_with_feed: Vec<TokenRecord>,
        swap: SwapParams,
    ) -> Result<()> {
        self.fixed.init(
            bump, store, nonce, market, user, position, params, tokens, senders, receivers,
        )?;
        self.prices = TokensWithFeed::try_from_vec(tokens_with_feed)?;
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
    /// Store.
    pub store: Pubkey,
    /// The nonce bytes for this order.
    pub nonce: [u8; 32],
    /// The slot that the order was last updated at.
    pub updated_at_slot: u64,
    /// The time that the order was last updated at.
    pub updated_at: i64,
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
        store: Pubkey,
        nonce: &NonceBytes,
        market: &Pubkey,
        user: &Pubkey,
        position: Option<&Pubkey>,
        params: &OrderParams,
        tokens: &Tokens,
        senders: &Senders,
        receivers: &Receivers,
    ) -> Result<()> {
        let clock = Clock::get()?;
        self.bump = bump;
        self.store = store;
        self.nonce = *nonce;
        self.updated_at_slot = clock.slot;
        self.updated_at = clock.unix_timestamp;
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
    /// Long token account.
    pub long_token_account: Pubkey,
    /// Short token account.
    pub short_token_account: Pubkey,
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

    /// Need to transfer in.
    pub fn need_to_transfer_in(&self) -> bool {
        matches!(self.kind, OrderKind::MarketIncrease | OrderKind::MarketSwap)
    }
}

/// Order Kind.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Copy)]
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

/// Transfer Out.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransferOut {
    /// Final output token.
    pub final_output_token: u64,
    /// Final secondary output token.
    pub final_secondary_output_token: u64,
    /// Long token.
    pub long_token: u64,
    /// Short token.
    pub short_token: u64,
    /// Long token amount for claimable account of user.
    pub long_token_for_claimable_account_of_user: u64,
    /// Short token amount for cliamable account of user.
    pub short_token_for_claimable_account_of_user: u64,
    /// Long token amount for claimable account of holding.
    pub long_token_for_claimable_account_of_holding: u64,
    /// Short token amount for claimable account of holding.
    pub short_token_for_claimable_account_of_holding: u64,
}

/// Recevier Kind.
pub enum CollateralReceiver {
    Funding,
    ClaimableForHolding,
    ClaimableForUser,
}

impl TransferOut {
    pub(crate) fn total_long_token_amount(&self) -> Result<u64> {
        self.long_token
            .checked_add(self.long_token_for_claimable_account_of_user)
            .and_then(|a| a.checked_add(self.long_token_for_claimable_account_of_holding))
            .ok_or(error!(DataStoreError::AmountOverflow))
    }

    pub(crate) fn total_short_token_amount(&self) -> Result<u64> {
        self.short_token
            .checked_add(self.short_token_for_claimable_account_of_user)
            .and_then(|a| a.checked_add(self.short_token_for_claimable_account_of_holding))
            .ok_or(error!(DataStoreError::AmountOverflow))
    }

    pub(crate) fn transfer_out(&mut self, is_secondary: bool, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        if is_secondary {
            self.final_secondary_output_token = self
                .final_secondary_output_token
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            self.final_output_token = self
                .final_output_token
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        Ok(())
    }

    pub(crate) fn transfer_out_funding_amounts(
        &mut self,
        long_amount: &u128,
        short_amount: &u128,
    ) -> Result<()> {
        self.transfer_out_collateral(
            true,
            CollateralReceiver::Funding,
            (*long_amount)
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            false,
            CollateralReceiver::Funding,
            (*short_amount)
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        Ok(())
    }

    pub(crate) fn process_claimable_collateral_for_decrease(
        &mut self,
        report: &DecreasePositionReport<u128>,
    ) -> Result<()> {
        let for_holding = report.claimable_collateral_for_holding();
        require!(
            *for_holding.output_token_amount() == 0,
            DataStoreError::ClaimbleCollateralInOutputTokenForHolding
        );

        let is_output_token_long = report.is_output_token_long();
        let is_secondary_token_long = report.is_secondary_output_token_long();

        let secondary_amount = (*for_holding.secondary_output_token_amount())
            .try_into()
            .map_err(|_| error!(DataStoreError::AmountOverflow))?;
        self.transfer_out_collateral(
            is_secondary_token_long,
            CollateralReceiver::ClaimableForHolding,
            secondary_amount,
        )?;

        let for_user = report.claimable_collateral_for_user();
        self.transfer_out_collateral(
            is_output_token_long,
            CollateralReceiver::ClaimableForUser,
            (*for_user.output_token_amount())
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            is_secondary_token_long,
            CollateralReceiver::ClaimableForUser,
            (*for_user.secondary_output_token_amount())
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        Ok(())
    }

    pub(crate) fn transfer_out_collateral(
        &mut self,
        is_long: bool,
        to: CollateralReceiver,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        match to {
            CollateralReceiver::Funding => {
                if is_long {
                    self.long_token = self
                        .long_token
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    self.short_token = self
                        .short_token
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForHolding => {
                if is_long {
                    self.long_token_for_claimable_account_of_holding = self
                        .long_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    self.short_token_for_claimable_account_of_holding = self
                        .short_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForUser => {
                if is_long {
                    self.long_token_for_claimable_account_of_user = self
                        .long_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    self.short_token_for_claimable_account_of_user = self
                        .short_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
        }
        Ok(())
    }
}
