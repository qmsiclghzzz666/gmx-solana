use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmx_core::{action::Prices, MarketExt, Position as _, PositionExt};

use crate::{
    constants,
    states::{
        order::{Order, OrderKind},
        position::Position,
        Config, DataStore, Market, Oracle, Roles, Seed, ValidateOracleTime,
    },
    utils::internal::{self, TransferUtils},
    DataStoreError, GmxCoreError,
};

use super::utils::swap::unchecked_swap_with_params;

#[derive(Accounts)]
pub struct ExecuteOrder<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_order_keeper: Account<'info, Roles>,
    #[account(
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
    pub oracle: Account<'info, Oracle>,
    #[account(
        constraint = order.fixed.market == market.key(),
        constraint = order.fixed.tokens.market_token == market_token_mint.key(),
        constraint = order.fixed.receivers.final_output_token_account == final_output_token_account.as_ref().map(|a| a.key()),
        constraint = order.fixed.receivers.secondary_output_token_account == secondary_output_token_account.as_ref().map(|a| a.key()),
    )]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub market_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = position.load()?.owner == order.fixed.user,
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
    pub final_output_token_vault: Option<Account<'info, TokenAccount>>,
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
    pub secondary_output_token_vault: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub final_output_token_account: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub secondary_output_token_account: Option<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

/// Execute an order.
pub fn execute_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
) -> Result<bool> {
    ctx.accounts.validate_time()?;
    // TODO: validate non-empty order.
    // TODO: validate order trigger price.
    ctx.accounts.pre_execute()?;
    let should_remove = ctx.accounts.execute(ctx.remaining_accounts)?;
    // TODO: validate market state.
    // TODO: emit order executed event.
    Ok(should_remove)
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

impl<'info> ValidateOracleTime for ExecuteOrder<'info> {
    fn oracle_updated_after(&self) -> Result<Option<i64>> {
        let ts = match self.order.fixed.params.kind {
            OrderKind::MarketSwap | OrderKind::MarketIncrease | OrderKind::MarketDecrease => {
                self.order.fixed.updated_at
            }
            OrderKind::Liquidation => {
                let position = self
                    .position
                    .as_ref()
                    .ok_or(error!(DataStoreError::PositionNotProvided))?
                    .load()?;
                position.increased_at.max(position.decreased_at)
            }
        };
        Ok(Some(ts))
    }

    fn oracle_updated_before(&self) -> Result<Option<i64>> {
        let ts = match self.order.fixed.params.kind {
            OrderKind::MarketIncrease | OrderKind::MarketDecrease => {
                Some(self.order.fixed.updated_at)
            }
            _ => None,
        };
        ts.map(|ts| self.config.request_expiration_at(ts))
            .transpose()
    }

    fn oracle_updated_after_slot(&self) -> Result<Option<u64>> {
        Ok(Some(self.order.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteOrder<'info> {
    fn validate_time(&self) -> Result<()> {
        self.oracle.validate_time(self)
    }

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

    fn pre_execute(&mut self) -> Result<()> {
        let report = self
            .market
            .as_market(&mut self.market_token_mint)
            .distribute_position_impact()
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(GmxCoreError::from)?;
        msg!("{:?}", report);
        Ok(())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<bool> {
        let prices = self.prices()?;
        let mut should_remove = false;
        let kind = &self.order.fixed.params.kind;
        match kind {
            OrderKind::MarketSwap => {
                unimplemented!();
            }
            OrderKind::MarketIncrease => {
                let Some(position) = self.position.as_mut() else {
                    return err!(DataStoreError::PositionNotProvided);
                };

                let params = &self.order.fixed.params;
                let swap = &self.order.swap;
                require!(
                    swap.short_token_swap_path.is_empty(),
                    DataStoreError::InvalidSwapPath
                );
                let collateral_token = position.load()?.collateral_token;

                // CHECK: no modification before, and `reload` has been called later.
                let (collateral_increment_amount, _) = unchecked_swap_with_params(
                    &self.oracle,
                    swap,
                    remaining_accounts,
                    (collateral_token, collateral_token),
                    (Some(self.order.fixed.tokens.initial_collateral_token), None),
                    (params.initial_collateral_delta_amount, 0),
                )?;
                // Call `reload` to make sure the state is up-to-date.
                self.market.reload()?;

                let size_delta_usd = params.size_delta_usd;
                let acceptable_price = params.acceptable_price;

                let report = self
                    .market
                    .as_market(&mut self.market_token_mint)
                    .into_position_ops(position)?
                    .increase(
                        prices,
                        collateral_increment_amount as u128,
                        size_delta_usd,
                        acceptable_price,
                    )
                    .map_err(GmxCoreError::from)?
                    .execute()
                    .map_err(GmxCoreError::from)?;
                msg!("{:?}", report);
            }
            OrderKind::MarketDecrease | OrderKind::Liquidation => {
                let Some(position) = self.position.as_mut() else {
                    return err!(DataStoreError::PositionNotProvided);
                };
                let params = &self.order.fixed.params;
                let collateral_withdrawal_amount = params.initial_collateral_delta_amount as u128;
                let size_delta_usd = params.size_delta_usd;
                let acceptable_price = params.acceptable_price;

                let (is_output_token_long, output_amount, secondary_output_amount) = {
                    let mut position = self
                        .market
                        .as_market(&mut self.market_token_mint)
                        .into_position_ops(position)?;

                    let report = position
                        .decrease(
                            prices,
                            size_delta_usd,
                            acceptable_price,
                            collateral_withdrawal_amount,
                            matches!(kind, OrderKind::Liquidation),
                            matches!(kind, OrderKind::Liquidation),
                        )
                        .map_err(GmxCoreError::from)?
                        .execute()
                        .map_err(GmxCoreError::from)?;
                    msg!("{:?}", report);

                    let mut output_amount = *report.output_amount();
                    let mut secondary_output_amount = *report.secondary_output_amount();
                    if secondary_output_amount != 0 {
                        require!(
                            report.is_output_token_long() != position.is_long(),
                            DataStoreError::SameSecondaryTokensNotMerged,
                        );
                        // Swap the secondary output tokens to output tokens.
                        let report = position
                            .into_market()
                            .swap(
                                !report.is_output_token_long(),
                                secondary_output_amount,
                                prices.long_token_price,
                                prices.short_token_price,
                            )
                            .map_err(GmxCoreError::from)?
                            .execute()
                            .map_err(GmxCoreError::from)?;
                        output_amount = output_amount
                            .checked_add(*report.token_out_amount())
                            .ok_or(DataStoreError::AmountOverflow)?;
                        secondary_output_amount = 0;
                    }
                    should_remove = report.should_remove();
                    (
                        report.is_output_token_long(),
                        output_amount
                            .try_into()
                            .map_err(|_| DataStoreError::AmountOverflow)?,
                        secondary_output_amount
                            .try_into()
                            .map_err(|_| DataStoreError::AmountOverflow)?,
                    )
                };

                // Swap output token to the expected output token.
                let meta = self.market.meta();
                let token_ins = if is_output_token_long {
                    (Some(meta.long_token_mint), Some(meta.short_token_mint))
                } else {
                    (Some(meta.short_token_mint), Some(meta.long_token_mint))
                };

                // Since we have checked that secondary_amount must be zero if output_token == secondary_output_token,
                // the swap should still be correct.

                // Call exit to make sure the data are written to the storage.
                // In case that there are markets also appear in the swap paths.
                self.market.exit(&crate::ID)?;
                // CHECK: `exit` and `reload` have been called on the modified market account before and after the swap.
                let (output_amount, secondary_amount) = unchecked_swap_with_params(
                    &self.oracle,
                    &self.order.swap,
                    remaining_accounts,
                    (
                        self.order
                            .fixed
                            .tokens
                            .final_output_token
                            .ok_or(DataStoreError::MissingTokenMint)?,
                        self.order.fixed.tokens.secondary_output_token,
                    ),
                    token_ins,
                    (output_amount, secondary_output_amount),
                )?;
                // Call `reload` to make sure the state is up-to-date.
                self.market.reload()?;

                self.transfer_out(false, output_amount)?;
                self.transfer_out(true, secondary_amount)?;
            }
        }
        Ok(should_remove)
    }

    fn transfer_out(&self, is_secondary: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_secondary {
            (
                &self.secondary_output_token_vault,
                &self.secondary_output_token_account,
            )
        } else {
            (
                &self.final_output_token_vault,
                &self.final_output_token_account,
            )
        };
        let (Some(from), Some(to)) = (from, to) else {
            return err!(DataStoreError::MissingReceivers);
        };
        TransferUtils::new(self.token_program.to_account_info(), &self.store, None).transfer_out(
            from.to_account_info(),
            to.to_account_info(),
            amount,
        )
    }
}
