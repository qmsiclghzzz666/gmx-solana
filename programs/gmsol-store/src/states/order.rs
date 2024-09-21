use anchor_lang::prelude::*;
use gmsol_model::action::decrease_position::DecreasePositionReport;

use crate::{CoreError, StoreError};

use super::{
    common::{
        action::{ActionHeader, ActionSigner},
        swap::SwapParamsV2,
        token::TokenAndAccount,
        SwapParams, TokenRecord, TokensWithFeed,
    },
    position::PositionKind,
    NonceBytes, Oracle, Seed,
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
        id: u64,
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
            bump, id, store, nonce, market, user, position, params, tokens, senders, receivers,
        )?;
        self.prices = TokensWithFeed::try_from_records(tokens_with_feed)?;
        self.swap = swap;
        Ok(())
    }

    /// Update order.
    pub fn update(&mut self, id: u64, update_params: &UpdateOrderParams) -> crate::Result<()> {
        let params = &mut self.fixed.params;
        params.validate_updatable()?;
        self.fixed.id = id;
        params.size_delta_usd = update_params.size_delta_usd;
        params.acceptable_price = update_params.acceptable_price;
        params.trigger_price = update_params.trigger_price;
        params.min_output_amount = update_params.min_output_amount;
        params.validate()?;

        self.fixed.updated()
    }

    pub(crate) fn validate_output_amount(&self, output_amount: u128) -> Result<()> {
        require_gte!(
            output_amount,
            self.fixed.params.min_output_amount,
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn validate_decrease_output_amounts(
        &self,
        oracle: &Oracle,
        output_token: &Pubkey,
        output_amount: u64,
        secondary_output_token: &Pubkey,
        secondary_output_amount: u64,
    ) -> Result<()> {
        let mut total = 0u128;
        {
            let price = oracle
                .primary
                .get(output_token)
                .ok_or(error!(StoreError::MissingOracelPrice))?
                .min
                .to_unit_price();
            let output_value = u128::from(output_amount).saturating_mul(price);
            total = total.saturating_add(output_value);
        }
        {
            let price = oracle
                .primary
                .get(secondary_output_token)
                .ok_or(error!(StoreError::MissingOracelPrice))?
                .min
                .to_unit_price();
            let output_value = u128::from(secondary_output_amount).saturating_mul(price);
            total = total.saturating_add(output_value);
        }

        // We use the `min_output_amount` as min output value.
        self.validate_output_amount(total)?;
        Ok(())
    }

    /// Get the params.
    pub fn params(&self) -> &OrderParams {
        &self.fixed.params
    }

    /// Validate trigger price.
    pub fn validate_trigger_price(&self, index_price: u128) -> Result<()> {
        let kind = &self.fixed.kind;
        let params = &self.fixed.params;
        let index_price = &index_price;
        match (kind, params.trigger_price.as_ref()) {
            (OrderKind::LimitIncrease, Some(trigger_price)) => {
                if params.is_long {
                    // TODO: Pick max price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick min price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                }
            }
            (OrderKind::LimitDecrease, Some(trigger_price)) => {
                if params.is_long {
                    // TODO: Pick min price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick max price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                }
            }
            (OrderKind::StopLossDecrease, Some(trigger_price)) => {
                if params.is_long {
                    // TODO: Pick min price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick max price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                }
            }
            (OrderKind::LimitSwap, _) => {
                // NOTE: For limit swap orders, the trigger price can be substituted by the min output amount,
                // so validatoin is not required. In fact, we should prohibit the creation of limit swap orders
                // with a trigger price.
            }
            (
                OrderKind::MarketSwap
                | OrderKind::MarketIncrease
                | OrderKind::MarketDecrease
                | OrderKind::Liquidation
                | OrderKind::AutoDeleveraging,
                _,
            ) => {}
            _ => {
                return err!(StoreError::InvalidTriggerPrice);
            }
        }

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
    /// Order Kind.
    pub kind: OrderKind,
    /// Store.
    pub store: Pubkey,
    /// The order market.
    pub market: Pubkey,
    /// Action id.
    pub id: u64,
    /// The slot that the order was last updated at.
    pub updated_at_slot: u64,
    /// The time that the order was last updated at.
    pub updated_at: i64,
    /// The bump seed.
    pub bump: u8,
    /// The nonce bytes for this order.
    pub nonce: [u8; 32],
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
    reserved: [u8; 128],
}

impl Fixed {
    #[allow(clippy::too_many_arguments)]
    fn init(
        &mut self,
        bump: u8,
        id: u64,
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
        self.bump = bump;
        self.id = id;
        self.kind = params.kind;
        self.store = store;
        self.nonce = *nonce;
        self.market = *market;
        self.user = *user;
        self.position = position.copied();
        self.params = params.clone();
        self.tokens = tokens.clone();
        self.senders = senders.clone();
        self.receivers = receivers.clone();
        self.updated()
    }

    fn updated(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        self.updated_at_slot = clock.slot;
        self.updated_at = clock.unix_timestamp;
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
    /// Minimum amount or value for output tokens.
    ///
    /// - Amount for swap orders.
    /// - Value for decrease position orders.
    pub min_output_amount: u128,
    /// Size delta usd.
    pub size_delta_usd: u128,
    /// Initial collateral delta amount.
    pub initial_collateral_delta_amount: u64,
    /// Trigger price (unit price).
    pub trigger_price: Option<u128>,
    /// Acceptable price (unit price).
    pub acceptable_price: Option<u128>,
    /// Whether the order is for a long or short position.
    pub is_long: bool,
}

impl OrderParams {
    /// Get position kind.
    pub fn to_position_kind(&self) -> Result<PositionKind> {
        if self.kind.is_swap() {
            Err(StoreError::PositionIsNotRequried.into())
        } else {
            Ok(if self.is_long {
                PositionKind::Long
            } else {
                PositionKind::Short
            })
        }
    }

    /// Need to transfer in.
    pub fn need_to_transfer_in(&self) -> bool {
        self.kind.is_increase_position() || self.kind.is_swap()
    }

    /// Return whether the order is updatable.
    pub fn is_updatable(&self) -> bool {
        matches!(
            self.kind,
            OrderKind::LimitIncrease
                | OrderKind::LimitDecrease
                | OrderKind::StopLossDecrease
                | OrderKind::LimitSwap
        )
    }

    /// Validate updatable.
    pub fn validate_updatable(&self) -> Result<()> {
        require!(self.is_updatable(), StoreError::InvalidArgument);
        Ok(())
    }

    /// Validate.
    pub fn validate(&self) -> Result<()> {
        match self.kind {
            OrderKind::MarketSwap
            | OrderKind::MarketIncrease
            | OrderKind::MarketDecrease
            | OrderKind::Liquidation
            | OrderKind::AutoDeleveraging => {
                require!(self.trigger_price.is_none(), StoreError::InvalidArgument);
            }
            OrderKind::LimitSwap => {
                // NOTE: The "trigger price" is replaced by the min output amount for limit swap orders.
                require!(self.trigger_price.is_none(), StoreError::InvalidArgument);
            }
            OrderKind::LimitIncrease | OrderKind::LimitDecrease | OrderKind::StopLossDecrease => {
                require!(self.trigger_price.is_some(), StoreError::InvalidArgument);
            }
        }
        // FIXME: should we validate for empty orders?
        Ok(())
    }
}

/// Update Order Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UpdateOrderParams {
    /// Size delta in USD.
    pub size_delta_usd: u128,
    /// Acceptable price.
    pub acceptable_price: Option<u128>,
    /// Trigger price.
    pub trigger_price: Option<u128>,
    /// Min output amount.
    pub min_output_amount: u128,
}

/// Order Kind.
#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Clone,
    InitSpace,
    Copy,
    strum::EnumString,
    strum::Display,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
)]
#[num_enum(error_type(name = CoreError, constructor = CoreError::unknown_order_kind))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
#[repr(u8)]
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
    /// Auto-deleveraging Order.
    AutoDeleveraging,
    /// Limit Swap.
    LimitSwap,
    /// Limit Increase.
    LimitIncrease,
    /// Limit Decrease.
    LimitDecrease,
    /// Stop-Loss Decrease.
    StopLossDecrease,
}

impl OrderKind {
    /// Is market order.
    pub fn is_market(&self) -> bool {
        matches!(
            self,
            Self::MarketSwap | Self::MarketIncrease | Self::MarketDecrease
        )
    }

    /// Is swap order.
    pub fn is_swap(&self) -> bool {
        matches!(self, Self::MarketSwap | Self::LimitSwap)
    }

    /// Is increase position order.
    pub fn is_increase_position(&self) -> bool {
        matches!(self, Self::LimitIncrease | Self::MarketIncrease)
    }

    /// Is decrease position order.
    pub fn is_decrease_position(&self) -> bool {
        matches!(
            self,
            Self::LimitDecrease
                | Self::MarketDecrease
                | Self::Liquidation
                | Self::AutoDeleveraging
                | Self::StopLossDecrease
        )
    }
}

/// Transfer Out.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransferOut {
    /// Executed.
    pub executed: bool,
    /// Final output token.
    pub final_output_token: u64,
    /// Secondary output token.
    pub secondary_output_token: u64,
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
    Collateral,
    ClaimableForHolding,
    ClaimableForUser,
}

impl TransferOut {
    /// Return whether the output for user is empty.
    pub fn is_user_output_empty(&self) -> bool {
        self.final_output_token == 0
            && self.secondary_output_token == 0
            && self.long_token == 0
            && self.short_token == 0
            && self.long_token_for_claimable_account_of_user == 0
            && self.short_token_for_claimable_account_of_user == 0
    }

    pub(crate) fn new_failed() -> Self {
        Self {
            executed: false,
            ..Default::default()
        }
    }

    pub(crate) fn total_long_token_amount(&self) -> Result<u64> {
        self.long_token
            .checked_add(self.long_token_for_claimable_account_of_user)
            .and_then(|a| a.checked_add(self.long_token_for_claimable_account_of_holding))
            .ok_or(error!(StoreError::AmountOverflow))
    }

    pub(crate) fn total_short_token_amount(&self) -> Result<u64> {
        self.short_token
            .checked_add(self.short_token_for_claimable_account_of_user)
            .and_then(|a| a.checked_add(self.short_token_for_claimable_account_of_holding))
            .ok_or(error!(StoreError::AmountOverflow))
    }

    pub(crate) fn transfer_out(&mut self, is_secondary: bool, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        if is_secondary {
            self.secondary_output_token = self
                .secondary_output_token
                .checked_add(amount)
                .ok_or(error!(StoreError::AmountOverflow))?;
        } else {
            self.final_output_token = self
                .final_output_token
                .checked_add(amount)
                .ok_or(error!(StoreError::AmountOverflow))?;
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
            CollateralReceiver::Collateral,
            (*long_amount)
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            false,
            CollateralReceiver::Collateral,
            (*short_amount)
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?,
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
            StoreError::ClaimbleCollateralInOutputTokenForHolding
        );

        let is_output_token_long = report.is_output_token_long();
        let is_secondary_token_long = report.is_secondary_output_token_long();

        let secondary_amount = (*for_holding.secondary_output_token_amount())
            .try_into()
            .map_err(|_| error!(StoreError::AmountOverflow))?;
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
                .map_err(|_| error!(StoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            is_secondary_token_long,
            CollateralReceiver::ClaimableForUser,
            (*for_user.secondary_output_token_amount())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?,
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
            CollateralReceiver::Collateral => {
                if is_long {
                    self.long_token = self
                        .long_token
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                } else {
                    self.short_token = self
                        .short_token
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForHolding => {
                if is_long {
                    self.long_token_for_claimable_account_of_holding = self
                        .long_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                } else {
                    self.short_token_for_claimable_account_of_holding = self
                        .short_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForUser => {
                if is_long {
                    self.long_token_for_claimable_account_of_user = self
                        .long_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                } else {
                    self.short_token_for_claimable_account_of_user = self
                        .short_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(StoreError::AmountOverflow))?;
                }
            }
        }
        Ok(())
    }
}

/// Order.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OrderV2 {
    /// Action header.
    pub(crate) header: ActionHeader,
    /// Market token.
    pub(crate) market_token: Pubkey,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Order params.
    pub(crate) params: OrderParamsV2,
    /// Max execution lamports.
    pub(crate) max_execution_lamports: u64,
    /// Swap params.
    pub(crate) swap: SwapParamsV2,
    padding_2: [u8; 4],
    pub(crate) updated_at: i64,
    pub(crate) updated_at_slot: u64,
    reserve: [u8; 128],
}

impl OrderV2 {
    /// Seed.
    pub const SEED: &'static [u8] = b"order";

    /// Init space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Min execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    /// Get signer.
    pub fn signer(&self) -> ActionSigner {
        self.header.signer(Self::SEED)
    }

    /// Validate trigger price.
    pub fn validate_trigger_price(&self, index_price: u128) -> Result<()> {
        let params = &self.params;
        let kind = params.kind()?;
        let index_price = &index_price;
        let is_long = params.side()?.is_long();
        let trigger_price = &params.trigger_price;
        match kind {
            OrderKind::LimitIncrease => {
                if is_long {
                    // TODO: Pick max price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick min price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                }
            }
            OrderKind::LimitDecrease => {
                if is_long {
                    // TODO: Pick min price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick max price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                }
            }
            OrderKind::StopLossDecrease => {
                if is_long {
                    // TODO: Pick min price.
                    require_gte!(trigger_price, index_price, StoreError::InvalidTriggerPrice);
                } else {
                    // TODO: Pick max price.
                    require_gte!(index_price, trigger_price, StoreError::InvalidTriggerPrice);
                }
            }
            OrderKind::LimitSwap => {
                // NOTE: For limit swap orders, the trigger price can be substituted by the min output amount,
                // so validatoin is not required. In fact, we should prohibit the creation of limit swap orders
                // with a trigger price.
            }

            OrderKind::MarketSwap
            | OrderKind::MarketIncrease
            | OrderKind::MarketDecrease
            | OrderKind::Liquidation
            | OrderKind::AutoDeleveraging => {}
        }

        Ok(())
    }

    /// Validate output amount.
    pub fn validate_output_amount(&self, output_amount: u128) -> Result<()> {
        require_gte!(
            output_amount,
            self.params.min_output,
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn validate_decrease_output_amounts(
        &self,
        oracle: &Oracle,
        output_token: &Pubkey,
        output_amount: u64,
        secondary_output_token: &Pubkey,
        secondary_output_amount: u64,
    ) -> Result<()> {
        let mut total = 0u128;
        {
            let price = oracle
                .primary
                .get(output_token)
                .ok_or(error!(StoreError::MissingOracelPrice))?
                .min
                .to_unit_price();
            let output_value = u128::from(output_amount).saturating_mul(price);
            total = total.saturating_add(output_value);
        }
        {
            let price = oracle
                .primary
                .get(secondary_output_token)
                .ok_or(error!(StoreError::MissingOracelPrice))?
                .min
                .to_unit_price();
            let output_value = u128::from(secondary_output_amount).saturating_mul(price);
            total = total.saturating_add(output_value);
        }

        // We use the `min_output_amount` as min output value.
        self.validate_output_amount(total)?;
        Ok(())
    }

    /// Get secondary output token (pnl token).
    pub fn secondary_output_token(&self) -> Result<Pubkey> {
        if self.params.side()?.is_long() {
            self.tokens.long_token.token()
        } else {
            self.tokens.short_token.token()
        }
        .ok_or(error!(CoreError::MissingPoolTokens))
    }
}

/// Token accounts for Order.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenAccounts {
    /// Initial collateral.
    pub(crate) initial_collateral: TokenAndAccount,
    /// Final output token.
    pub(crate) final_output_token: TokenAndAccount,
    /// Long token.
    pub(crate) long_token: TokenAndAccount,
    /// Short token.
    pub(crate) short_token: TokenAndAccount,
}

/// Order params.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OrderParamsV2 {
    /// Kind.
    kind: u8,
    /// Order side.
    side: u8,
    padding_1: [u8; 6],
    /// Collateral/Output token.
    collateral_token: Pubkey,
    /// Position address.
    position: Pubkey,
    /// Initial collateral delta amount.
    pub(crate) initial_collateral_delta_amount: u64,
    /// Size delta value.
    pub(crate) size_delta_value: u128,
    /// Min output amount or value.
    /// - Used as amount for swap orders.
    /// - Used as value for decrease position orders.
    min_output: u128,
    /// Trigger price (in unit price).
    pub(crate) trigger_price: u128,
    /// Acceptable price (in unit price).
    pub(crate) acceptable_price: u128,
    reserve: [u8; 128],
}

impl OrderParamsV2 {
    pub(crate) fn init_swap(
        &mut self,
        kind: OrderKind,
        collateral_token: Pubkey,
        swap_in_amount: u64,
        min_output: Option<u128>,
    ) -> Result<()> {
        self.kind = kind.into();
        self.collateral_token = collateral_token;
        self.initial_collateral_delta_amount = swap_in_amount;
        match kind {
            OrderKind::MarketSwap => {
                self.min_output = min_output.unwrap_or(0);
            }
            OrderKind::LimitSwap => {
                let Some(min_output) = min_output else {
                    return err!(CoreError::InvalidMinOutputAmount);
                };
                require!(min_output != 0, CoreError::Internal);
                self.min_output = min_output;
            }
            _ => {
                return err!(CoreError::Internal);
            }
        }
        Ok(())
    }

    pub(crate) fn init_increase(
        &mut self,
        is_long: bool,
        kind: OrderKind,
        collateral_token: Pubkey,
        initial_collateral_delta_amount: u64,
        trigger_price: Option<u128>,
        acceptable_price: Option<u128>,
    ) -> Result<()> {
        self.kind = kind.into();
        self.collateral_token = collateral_token;
        self.initial_collateral_delta_amount = initial_collateral_delta_amount;
        match acceptable_price {
            Some(price) => {
                self.acceptable_price = price;
            }
            None => {
                if is_long {
                    self.acceptable_price = u128::MAX;
                } else {
                    self.acceptable_price = u128::MIN;
                }
            }
        }
        match kind {
            OrderKind::MarketIncrease => {
                require!(trigger_price.is_none(), CoreError::InvalidTriggerPrice);
            }
            OrderKind::LimitIncrease => {
                let Some(price) = trigger_price else {
                    return err!(CoreError::InvalidTriggerPrice);
                };
                self.trigger_price = price;
            }
            _ => {
                return err!(CoreError::Internal);
            }
        }
        Ok(())
    }

    pub(crate) fn init_decrease(
        &mut self,
        is_long: bool,
        kind: OrderKind,
        collateral_token: Pubkey,
        initial_collateral_delta_amount: u64,
        trigger_price: Option<u128>,
        acceptable_price: Option<u128>,
    ) -> Result<()> {
        self.kind = kind.into();
        self.collateral_token = collateral_token;
        self.initial_collateral_delta_amount = initial_collateral_delta_amount;
        match acceptable_price {
            Some(price) => {
                self.acceptable_price = price;
            }
            None => {
                if is_long {
                    self.acceptable_price = u128::MIN;
                } else {
                    self.acceptable_price = u128::MAX;
                }
            }
        }
        match kind {
            OrderKind::MarketDecrease | OrderKind::Liquidation | OrderKind::AutoDeleveraging => {
                require!(trigger_price.is_none(), CoreError::InvalidTriggerPrice);
            }
            OrderKind::LimitDecrease | OrderKind::StopLossDecrease => {
                let Some(price) = trigger_price else {
                    return err!(CoreError::InvalidTriggerPrice);
                };
                self.trigger_price = price;
            }
            _ => {
                return err!(CoreError::Internal);
            }
        }
        Ok(())
    }

    /// Get order kind.
    pub fn kind(&self) -> Result<OrderKind> {
        Ok(self.kind.try_into()?)
    }

    /// Get order side.
    pub fn side(&self) -> Result<OrderSide> {
        let side = self.side.try_into()?;
        Ok(side)
    }

    /// Get position address.
    pub fn position(&self) -> Option<&Pubkey> {
        if self.position != Pubkey::default() {
            Some(&self.position)
        } else {
            None
        }
    }
}

/// Order side.
#[derive(
    Clone,
    Copy,
    strum::EnumString,
    strum::Display,
    num_enum::IntoPrimitive,
    num_enum::TryFromPrimitive,
)]
#[num_enum(error_type(name = CoreError, constructor = CoreError::unknown_order_side))]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
#[repr(u8)]
pub enum OrderSide {
    /// Long.
    Long,
    /// Short.
    Short,
}

impl OrderSide {
    /// Return whether the side is long.
    pub fn is_long(&self) -> bool {
        matches!(self, Self::Long)
    }
}
