use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gmx_core::{action::Prices, PositionExt};

use crate::{
    states::{
        order::{Order, OrderKind},
        position::Position,
        DataStore, Market, Oracle, Roles,
    },
    utils::internal,
    DataStoreError, GmxCoreError,
};

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_order_keeper: Account<'info, Roles>,
    pub oracle: Account<'info, Oracle>,
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub position: Option<AccountLoader<'info, Position>>,
}

/// Execute an order.
pub fn execute_order<'info>(ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>) -> Result<()> {
    // TODO: validate non-empty order.
    // TODO: validate order trigger price.
    ctx.accounts.execute()?;
    // TODO: validate market state.
    // TODO: emit order executed event.
    Ok(())
}

impl<'info> internal::Authentication<'info> for ExecuteOrder<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_order_keeper
    }
}

impl<'info> ExecuteOrder<'info> {
    fn prices(&self) -> Result<Prices<u128>> {
        let meta = self.market.meta();
        let oracle = &self.oracle.primary;
        Ok(Prices {
            index_token_price: oracle
                .get(&meta.index_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
            long_token_price: oracle
                .get(&meta.long_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
            short_token_price: oracle
                .get(&meta.short_token_mint)
                .ok_or(DataStoreError::MissingOracelPrice)?
                .max
                .to_unit_price(),
        })
    }

    fn execute(&mut self) -> Result<()> {
        let prices = self.prices()?;
        match self.order.fixed.params.kind {
            OrderKind::MarketSwap => {
                unimplemented!();
            }
            OrderKind::MarketIncrease => {
                let Some(position) = self.position.as_mut() else {
                    return err!(DataStoreError::PositionNotProvided);
                };
                {
                    let params = &self.order.fixed.params;
                    let collateral_increment_amount =
                        params.initial_collateral_delta_amount as u128;
                    let size_delta_usd = params.size_delta_usd;
                    let acceptable_price = params.acceptable_price;
                    let mut position = self
                        .market
                        .as_market(&self.market_token_mint)
                        .into_position_ops(position)?;

                    let report = position
                        .increase(
                            prices,
                            collateral_increment_amount,
                            size_delta_usd,
                            Some(acceptable_price),
                        )
                        .map_err(GmxCoreError::from)?
                        .execute()
                        .map_err(GmxCoreError::from)?;
                    msg!("{:?}", report);
                    // TODO: swap and transfer.
                }
            }
            OrderKind::MarketDecrease => {
                let Some(position) = self.position.as_mut() else {
                    return err!(DataStoreError::PositionNotProvided);
                };
                {
                    let params = &self.order.fixed.params;
                    let collateral_withdrawal_amount =
                        params.initial_collateral_delta_amount as u128;
                    let size_delta_usd = params.size_delta_usd;
                    let acceptable_price = params.acceptable_price;
                    let mut position = self
                        .market
                        .as_market(&self.market_token_mint)
                        .into_position_ops(position)?;

                    let report = position
                        .decrease(
                            prices,
                            size_delta_usd,
                            Some(acceptable_price),
                            collateral_withdrawal_amount,
                        )
                        .map_err(GmxCoreError::from)?
                        .execute()
                        .map_err(GmxCoreError::from)?;
                    msg!("{:?}", report);
                    // TODO: swap and transfer.
                }
            }
            OrderKind::Liquidation => {
                unimplemented!();
            }
        }
        Ok(())
    }
}
