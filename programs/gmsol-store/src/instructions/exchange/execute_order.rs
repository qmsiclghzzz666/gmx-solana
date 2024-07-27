use std::ops::Deref;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_model::{
    action::Prices, num::Unsigned, BaseMarket, BaseMarketExt, PnlFactorKind, Position as _,
    PositionExt, PositionImpactMarketExt,
};

use crate::{
    constants,
    events::TradeEvent,
    states::{
        ops::{AdlOps, ValidateMarketBalances},
        order::{CollateralReceiver, Order, OrderKind, TransferOut},
        position::Position,
        revertible::{
            perp_market::RevertiblePerpMarket,
            revertible_position::RevertiblePosition,
            swap_market::{SwapDirection, SwapMarkets},
            Revertible,
        },
        HasMarketMeta, Market, Oracle, Seed, Store, ValidateOracleTime,
    },
    utils::internal,
    ModelError, StoreError, StoreResult,
};

type ShouldRemovePosition = bool;

#[event_cpi]
#[derive(Accounts)]
#[instruction(recent_timestamp: i64)]
pub struct ExecuteOrder<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub oracle: Box<Account<'info, Oracle>>,
    #[account(
        mut,
        constraint = order.fixed.store == store.key(),
        constraint = order.fixed.market == market.key(),
        constraint = order.fixed.tokens.market_token == market_token_mint.key(),
        constraint = order.fixed.receivers.final_output_token_account == final_output_token_account.as_ref().map(|a| a.key()),
        constraint = order.fixed.receivers.secondary_output_token_account == secondary_output_token_account.as_ref().map(|a| a.key()),
        constraint = order.fixed.receivers.long_token_account == long_token_account.key(),
        constraint = order.fixed.receivers.short_token_account == short_token_account.key(),
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    pub market_token_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = position.load()?.owner == order.fixed.user,
        constraint = position.load()?.store == store.key(),
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            order.fixed.user.as_ref(),
            position.load()?.market_token.as_ref(),
            position.load()?.collateral_token.as_ref(),
            &[position.load()?.kind],
        ],
        bump = position.load()?.bump,
    )]
    pub position: Option<AccountLoader<'info, Position>>,
    #[account(
        mut,
        token::mint = final_output_token_account.as_ref().expect("must provided").mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_output_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_output_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = secondary_output_token_account.as_ref().expect("must provided").mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            secondary_output_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub secondary_output_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut)]
    pub final_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut)]
    pub secondary_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = market.load()?.meta().long_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market.load()?.meta().long_token_mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub long_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = market.load()?.meta().short_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market.load()?.meta().short_token_mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub short_token_vault: Account<'info, TokenAccount>,
    /// CHECK: check by token program.
    #[account(mut)]
    pub long_token_account: UncheckedAccount<'info>,
    /// CHECK: check by token program.
    #[account(mut)]
    pub short_token_account: UncheckedAccount<'info>,
    #[account(
        mut,
        token::mint = market.load()?.meta().long_token_mint,
        token::authority = store,
        constraint = check_delegation(claimable_long_token_account_for_user, order.fixed.user)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            market.load()?.meta().long_token_mint.as_ref(),
            order.fixed.user.as_ref(),
            &store.load()?.claimable_time_key(validated_recent_timestamp(store.load()?.deref(), recent_timestamp)?)?,
        ],
        bump,
    )]
    pub claimable_long_token_account_for_user: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = market.load()?.meta().short_token_mint,
        token::authority = store,
        constraint = check_delegation(claimable_short_token_account_for_user, order.fixed.user)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            market.load()?.meta().short_token_mint.as_ref(),
            order.fixed.user.as_ref(),
            &store.load()?.claimable_time_key(validated_recent_timestamp(store.load()?.deref(), recent_timestamp)?)?,
        ],
        bump,
    )]
    pub claimable_short_token_account_for_user: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = get_pnl_token(&position, market.load()?.deref())?,
        token::authority = store,
        constraint = check_delegation(claimable_pnl_token_account_for_holding, store.load()?.address.holding)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            get_pnl_token(&position, market.load()?.deref())?.as_ref(),
            store.load()?.address.holding.as_ref(),
            &store.load()?.claimable_time_key(validated_recent_timestamp(store.load()?.deref(), recent_timestamp)?)?,
        ],
        bump,
    )]
    pub claimable_pnl_token_account_for_holding: Option<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

/// Execute an order.
pub fn execute_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
    _recent_timestamp: i64,
    throw_on_execution_error: bool,
) -> Result<(ShouldRemovePosition, Box<TransferOut>)> {
    match ctx.accounts.validate_oracle() {
        Ok(()) => {}
        Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
            msg!(
                "Order expired at {}",
                ctx.accounts
                    .oracle_updated_before()
                    .ok()
                    .flatten()
                    .expect("must have an expiration time"),
            );
            return Ok((false, Box::new(TransferOut::new_failed())));
        }
        Err(err) => {
            return Err(error!(err));
        }
    }
    let prices = ctx.accounts.prices()?;
    match ctx.accounts.execute(prices, ctx.remaining_accounts) {
        Ok((should_remove_position, mut transfer_out, trade_event)) => {
            if let Some(event) = trade_event {
                emit_cpi!(event);
            }
            transfer_out.executed = true;
            Ok((should_remove_position, transfer_out))
        }
        Err(err) if !throw_on_execution_error => {
            msg!("Execute order error: {}", err);
            Ok((false, Default::default()))
        }
        Err(err) => Err(err),
    }
}

impl<'info> internal::Authentication<'info> for ExecuteOrder<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ValidateOracleTime for ExecuteOrder<'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        match self.order.fixed.params.kind {
            OrderKind::MarketSwap
            | OrderKind::LimitSwap
            | OrderKind::MarketIncrease
            | OrderKind::MarketDecrease
            | OrderKind::LimitIncrease => Ok(Some(self.order.fixed.updated_at)),
            OrderKind::LimitDecrease | OrderKind::StopLossDecrease => {
                let position = self
                    .position
                    .as_ref()
                    .ok_or(StoreError::PositionNotProvided)?
                    .load()
                    .map_err(|_| StoreError::LoadAccountError)?;
                let last_updated = self.order.fixed.updated_at.max(position.state.increased_at);
                Ok(Some(last_updated))
            }
            OrderKind::Liquidation => {
                let position = self
                    .position
                    .as_ref()
                    .ok_or(StoreError::PositionNotProvided)?
                    .load()
                    .map_err(|_| StoreError::LoadAccountError)?;
                Ok(Some(
                    position.state.increased_at.max(position.state.decreased_at),
                ))
            }
            // Ignore the check of oracle ts for ADL orders.
            OrderKind::AutoDeleveraging => Ok(None),
        }
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = match self.order.fixed.params.kind {
            OrderKind::MarketSwap | OrderKind::MarketIncrease | OrderKind::MarketDecrease => {
                Some(self.order.fixed.updated_at)
            }
            _ => None,
        };
        ts.map(|ts| {
            self.store
                .load()
                .map_err(|_| StoreError::LoadAccountError)?
                .request_expiration_at(ts)
        })
        .transpose()
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        // FIXME: should we validate the slot for liquidation and ADL?
        let after = match self.order.fixed.params.kind {
            OrderKind::Liquidation | OrderKind::AutoDeleveraging => None,
            _ => Some(self.order.fixed.updated_at_slot),
        };
        Ok(after)
    }
}

impl<'info> ExecuteOrder<'info> {
    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)?;
        #[allow(clippy::single_match)]
        match &self.order.fixed.kind {
            OrderKind::AutoDeleveraging => {
                self.market
                    .load()
                    .map_err(|_| StoreError::InvalidMarket)?
                    .validate_adl(&self.oracle, self.order.fixed.params.is_long)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())?;
        Ok(())
    }

    fn validate_order(&self, prices: &Prices<u128>) -> Result<()> {
        self.validate_non_empty_order()?;
        self.validate_trigger_price(prices)?;
        Ok(())
    }

    fn validate_non_empty_order(&self) -> Result<()> {
        let order = &self.order;
        let kind = &order.fixed.kind;
        let params = &order.fixed.params;

        // NOTE: we currently allow the delta size for decrease position order to be empty.
        if kind.is_increase_position() {
            require!(params.size_delta_usd != 0, StoreError::InvalidArgument);
        }

        if kind.is_swap() {
            require!(
                params.initial_collateral_delta_amount != 0,
                StoreError::InvalidArgument
            );
        }
        Ok(())
    }

    fn validate_trigger_price(&self, prices: &Prices<u128>) -> Result<()> {
        let order = &self.order;
        let kind = &order.fixed.kind;
        let params = &order.fixed.params;
        let index_price = &prices.index_token_price;
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
                // NOTE: we leave the check of the trigger price for limit swap to the execution part.
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

    #[inline(never)]
    fn execute(
        &mut self,
        prices: Prices<u128>,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(ShouldRemovePosition, Box<TransferOut>, Option<TradeEvent>)> {
        self.validate_market()?;
        self.validate_order(&prices)?;

        // Prepare execution context.
        let mut market = RevertiblePerpMarket::new(&self.market)?;
        let current_market_token = self.market_token_mint.key();
        let loaders = self
            .order
            .swap
            .unpack_markets_for_swap(&current_market_token, remaining_accounts)?;
        let mut swap_markets =
            SwapMarkets::new(&self.store.key(), &loaders, Some(&current_market_token))?;
        let mut transfer_out = Box::default();

        // Distribute position impact.
        {
            let report = market
                .distribute_position_impact()
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Order] pre-execute: {:?}", report);
        }

        let kind = self.order.fixed.params.kind;
        let mut trade_event = None;
        let should_remove_position = match &kind {
            OrderKind::MarketSwap | OrderKind::LimitSwap => {
                execute_swap(
                    &self.oracle,
                    &mut market,
                    &mut swap_markets,
                    &mut transfer_out,
                    &mut self.order,
                )?;
                market.commit();
                false
            }
            OrderKind::MarketIncrease
            | OrderKind::MarketDecrease
            | OrderKind::Liquidation
            | OrderKind::AutoDeleveraging
            | OrderKind::LimitIncrease
            | OrderKind::LimitDecrease
            | OrderKind::StopLossDecrease => {
                let position_loader = self
                    .position
                    .as_ref()
                    .ok_or(error!(StoreError::PositionIsNotProvided))?;
                let mut event = TradeEvent::new_unchanged(
                    matches!(kind, OrderKind::MarketIncrease),
                    position_loader.key(),
                    &*position_loader.load()?,
                    self.order.key(),
                )?;
                let mut position = RevertiblePosition::new(market, position_loader)?;

                let should_remove_position = match kind {
                    OrderKind::MarketIncrease | OrderKind::LimitIncrease => {
                        execute_increase_position(
                            &self.oracle,
                            prices,
                            &mut position,
                            &mut swap_markets,
                            &mut transfer_out,
                            &mut event,
                            &mut self.order,
                        )?;
                        false
                    }
                    OrderKind::Liquidation => execute_decrease_position(
                        &self.oracle,
                        prices,
                        &mut position,
                        &mut swap_markets,
                        &mut transfer_out,
                        &mut event,
                        &mut self.order,
                        true,
                        Some(SecondaryOrderType::Liquidation),
                    )?,
                    OrderKind::AutoDeleveraging => execute_decrease_position(
                        &self.oracle,
                        prices,
                        &mut position,
                        &mut swap_markets,
                        &mut transfer_out,
                        &mut event,
                        &mut self.order,
                        true,
                        Some(SecondaryOrderType::AutoDeleveraging),
                    )?,
                    OrderKind::MarketDecrease
                    | OrderKind::LimitDecrease
                    | OrderKind::StopLossDecrease => execute_decrease_position(
                        &self.oracle,
                        prices,
                        &mut position,
                        &mut swap_markets,
                        &mut transfer_out,
                        &mut event,
                        &mut self.order,
                        false,
                        None,
                    )?,
                    _ => unreachable!(),
                };
                position.write_to_event(&mut event)?;
                event.update_with_transfer_out(&transfer_out)?;
                trade_event = Some(event);
                position.commit();
                msg!(
                    "[Position] executed with trade_id={}",
                    self.position
                        .as_ref()
                        .unwrap()
                        .load()
                        .unwrap()
                        .state
                        .trade_id
                );
                should_remove_position
            }
        };
        swap_markets.commit();
        Ok((should_remove_position, transfer_out, trade_event))
    }

    fn prices(&self) -> Result<Prices<u128>> {
        self.market.load()?.prices(&self.oracle)
    }
}

fn execute_swap(
    oracle: &Oracle,
    market: &mut RevertiblePerpMarket<'_>,
    swap_markets: &mut SwapMarkets<'_>,
    transfer_out: &mut TransferOut,
    order: &mut Order,
) -> Result<()> {
    let swap_out_token = order.fixed.tokens.output_token;
    // Perform swap.
    let swap_out_amount = {
        let swap = &order.swap;
        require!(
            swap.short_token_swap_path.is_empty(),
            StoreError::InvalidSwapPath
        );
        let (swap_out_amount, _) = swap_markets.revertible_swap(
            SwapDirection::Into(market),
            oracle,
            swap,
            (swap_out_token, swap_out_token),
            (Some(order.fixed.tokens.initial_collateral_token), None),
            (order.fixed.params.initial_collateral_delta_amount, 0),
        )?;
        swap_out_amount
    };
    require_gte!(
        u128::from(swap_out_amount),
        order.fixed.params.min_output_amount,
        StoreError::InsufficientOutputAmount
    );
    let is_long = market.market_meta().to_token_side(&swap_out_token)?;
    transfer_out.transfer_out_collateral(
        is_long,
        CollateralReceiver::Collateral,
        swap_out_amount,
    )?;
    order.fixed.params.initial_collateral_delta_amount = 0;
    Ok(())
}

fn execute_increase_position(
    oracle: &Oracle,
    prices: Prices<u128>,
    position: &mut RevertiblePosition<'_>,
    swap_markets: &mut SwapMarkets<'_>,
    transfer_out: &mut TransferOut,
    event: &mut TradeEvent,
    order: &mut Order,
) -> Result<()> {
    let params = &order.fixed.params;

    // Perform swap.
    let collateral_increment_amount = {
        let swap = &order.swap;
        require!(
            swap.short_token_swap_path.is_empty(),
            StoreError::InvalidSwapPath
        );
        let collateral_token = *position.collateral_token();
        let (collateral_increment_amount, _) = swap_markets.revertible_swap(
            SwapDirection::Into(position.market_mut()),
            oracle,
            swap,
            (collateral_token, collateral_token),
            (Some(order.fixed.tokens.initial_collateral_token), None),
            (params.initial_collateral_delta_amount, 0),
        )?;
        collateral_increment_amount
    };

    // Increase position.
    let (long_amount, short_amount) = {
        let size_delta_usd = params.size_delta_usd;
        let acceptable_price = params.acceptable_price;
        let report = position
            .increase(
                prices,
                collateral_increment_amount.into(),
                size_delta_usd,
                acceptable_price,
            )
            .and_then(|a| a.execute())
            .map_err(ModelError::from)?;
        msg!("[Position] increased: {:?}", report);
        let (long_amount, short_amount) = report.claimable_funding_amounts();
        event.update_with_increase_report(&report)?;
        (*long_amount, *short_amount)
    };

    // Process output amount.
    transfer_out.transfer_out_funding_amounts(&long_amount, &short_amount)?;

    position.market().validate_market_balances(
        long_amount
            .try_into()
            .map_err(|_| error!(StoreError::AmountOverflow))?,
        short_amount
            .try_into()
            .map_err(|_| error!(StoreError::AmountOverflow))?,
    )?;

    order.fixed.params.initial_collateral_delta_amount = 0;
    Ok(())
}

enum SecondaryOrderType {
    Liquidation,
    AutoDeleveraging,
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn execute_decrease_position(
    oracle: &Oracle,
    prices: Prices<u128>,
    position: &mut RevertiblePosition<'_>,
    swap_markets: &mut SwapMarkets<'_>,
    transfer_out: &mut TransferOut,
    event: &mut TradeEvent,
    order: &mut Order,
    is_insolvent_close_allowed: bool,
    secondary_order_type: Option<SecondaryOrderType>,
) -> Result<ShouldRemovePosition> {
    // Decrease position.
    let report = {
        let params = &order.fixed.params;
        let collateral_withdrawal_amount = params.initial_collateral_delta_amount as u128;
        let size_delta_usd = params.size_delta_usd;
        let acceptable_price = params.acceptable_price;
        let is_liquidation_order =
            matches!(secondary_order_type, Some(SecondaryOrderType::Liquidation));
        let is_adl_order = matches!(
            secondary_order_type,
            Some(SecondaryOrderType::AutoDeleveraging)
        );
        // Only required when the order is an ADL order.
        let mut pnl_factor_before_execution = None;

        // Validate the liqudiation is a fully close.
        if is_liquidation_order {
            require_gte!(
                size_delta_usd,
                *position.size_in_usd(),
                StoreError::InvalidArgument
            );
        }

        // Validate that ADL is required.
        if is_adl_order {
            let Some((pnl_factor, _)) = position
                .market()
                .pnl_factor_exceeded(&prices, PnlFactorKind::ForAdl, params.is_long)
                .map_err(ModelError::from)?
            else {
                return err!(StoreError::AdlNotRequired);
            };
            pnl_factor_before_execution = Some(pnl_factor);
        }

        let report = position
            .decrease(
                prices,
                size_delta_usd,
                acceptable_price,
                collateral_withdrawal_amount,
                is_insolvent_close_allowed,
                is_liquidation_order,
            )
            .and_then(|a| a.execute())
            .map_err(ModelError::from)?;

        // Validate that ADL is valid.
        if is_adl_order {
            let pnl_factor_after_execution = position
                .market()
                .pnl_factor(&prices, params.is_long, true)
                .map_err(ModelError::from)?;
            require_gt!(
                pnl_factor_before_execution.expect("must be some"),
                pnl_factor_after_execution,
                StoreError::InvalidAdl
            );
            let min_pnl_factor = position
                .market()
                .pnl_factor_config(PnlFactorKind::MinAfterAdl, params.is_long)
                .and_then(|factor| factor.to_signed())
                .map_err(ModelError::from)?;
            require_gt!(
                pnl_factor_after_execution,
                min_pnl_factor,
                StoreError::InvalidAdl
            );
        }

        msg!("[Position] decreased: {:?}", report);
        event.update_with_decrease_report(&report)?;
        report
    };
    let should_remove_position = report.should_remove();

    // Perform swaps.
    {
        require!(
            *report.secondary_output_amount() == 0
                || (report.is_output_token_long() != report.is_secondary_output_token_long()),
            StoreError::SameSecondaryTokensNotMerged,
        );
        let (is_output_token_long, output_amount, secondary_output_amount) = (
            report.is_output_token_long(),
            (*report.output_amount())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?,
            (*report.secondary_output_amount())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?,
        );

        // Swap output token to the expected output token.
        let meta = *position.market().market_meta();
        let token_ins = if is_output_token_long {
            (Some(meta.long_token_mint), Some(meta.short_token_mint))
        } else {
            (Some(meta.short_token_mint), Some(meta.long_token_mint))
        };

        // Since we have checked that secondary_amount must be zero if output_token == secondary_output_token,
        // the swap should still be correct.

        let final_output_token = order
            .fixed
            .tokens
            .final_output_token
            .ok_or(error!(StoreError::MissingTokenMint))?;
        let secondary_output_token = order.fixed.tokens.secondary_output_token;
        let (output_amount, secondary_output_amount) = swap_markets.revertible_swap(
            SwapDirection::From(position.market_mut()),
            oracle,
            &order.swap,
            (final_output_token, secondary_output_token),
            token_ins,
            (output_amount, secondary_output_amount),
        )?;
        validate_output_amount(
            oracle,
            &final_output_token,
            output_amount,
            &secondary_output_token,
            secondary_output_amount,
            order.fixed.params.min_output_amount,
        )?;
        transfer_out.transfer_out(false, output_amount)?;
        transfer_out.transfer_out(true, secondary_output_amount)?;
    }

    // Process other output amounts.
    {
        let (long_amount, short_amount) = report.claimable_funding_amounts();
        transfer_out.transfer_out_funding_amounts(long_amount, short_amount)?;
        transfer_out.process_claimable_collateral_for_decrease(&report)?;
    }

    // Validate market balances.
    let mut long_transfer_out = transfer_out.total_long_token_amount()?;
    let mut short_transfer_out = transfer_out.total_short_token_amount()?;
    let mut add_to_amount = |is_long_token: bool, amount: u64| {
        let acc = if is_long_token {
            &mut long_transfer_out
        } else {
            &mut short_transfer_out
        };
        *acc = acc
            .checked_add(amount)
            .ok_or(error!(StoreError::AmountOverflow))?;
        Result::Ok(())
    };
    let current_market_token = position.market().key();
    let meta = position.market().market_meta();
    let tokens = &order.fixed.tokens;
    let output_token_market = order
        .swap
        .last_market_token(true)
        .unwrap_or(&current_market_token);
    let secondary_token_market = order
        .swap
        .last_market_token(false)
        .unwrap_or(&current_market_token);
    if transfer_out.final_output_token != 0 && *output_token_market == current_market_token {
        (add_to_amount)(
            meta.to_token_side(
                tokens
                    .final_output_token
                    .as_ref()
                    .ok_or(error!(StoreError::InvalidArgument))?,
            )?,
            transfer_out.final_output_token,
        )?;
    }
    if transfer_out.final_secondary_output_token != 0
        && *secondary_token_market == current_market_token
    {
        (add_to_amount)(
            meta.to_token_side(&tokens.secondary_output_token)?,
            transfer_out.final_secondary_output_token,
        )?;
    }
    position
        .market()
        .validate_market_balances(long_transfer_out, short_transfer_out)?;

    Ok(should_remove_position)
}

fn get_pnl_token(
    position: &Option<AccountLoader<'_, Position>>,
    market: &Market,
) -> Result<Pubkey> {
    let is_long = position
        .as_ref()
        .ok_or(error!(StoreError::MissingPosition))?
        .load()?
        .is_long()?;
    if is_long {
        Ok(market.meta().long_token_mint)
    } else {
        Ok(market.meta.short_token_mint)
    }
}

fn check_delegation(account: &TokenAccount, target: Pubkey) -> Result<bool> {
    let is_matched = account
        .delegate
        .map(|delegate| delegate == target)
        .ok_or(error!(StoreError::NoDelegatedAuthorityIsSet))?;
    Ok(is_matched)
}

fn validated_recent_timestamp(config: &Store, timestamp: i64) -> Result<i64> {
    let recent_time_window = config.amount.recent_time_window;
    let expiration_time = timestamp.saturating_add_unsigned(recent_time_window);
    let clock = Clock::get()?;
    if timestamp <= clock.unix_timestamp && clock.unix_timestamp <= expiration_time {
        Ok(timestamp)
    } else {
        err!(StoreError::InvalidArgument)
    }
}

#[inline(never)]
fn validate_output_amount(
    oracle: &Oracle,
    output_token: &Pubkey,
    output_amount: u64,
    secondary_output_token: &Pubkey,
    secondary_output_amount: u64,
    min_output_value: u128,
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

    require_gte!(
        total,
        min_output_value,
        StoreError::InsufficientOutputAmount
    );
    Ok(())
}
