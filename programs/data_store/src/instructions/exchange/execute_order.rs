use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmx_core::{
    action::{
        decrease_position::DecreasePositionReport, increase_position::IncreasePositionReport,
        Prices,
    },
    MarketExt, PositionExt,
};

use crate::{
    constants,
    states::{
        order::{Order, OrderKind, TransferOut},
        position::Position,
        Config, DataStore, Market, Oracle, Roles, Seed, ValidateOracleTime,
    },
    utils::internal::{self},
    DataStoreError, GmxCoreError,
};

use super::utils::swap::unchecked_swap_with_params;

#[derive(Accounts)]
#[instruction(recent_timestamp: i64)]
pub struct ExecuteOrder<'info> {
    pub authority: Signer<'info>,
    pub store: Box<Account<'info, DataStore>>,
    pub only_order_keeper: Account<'info, Roles>,
    #[account(
        has_one = store,
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Box<Account<'info, Config>>,
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
    pub market: Box<Account<'info, Market>>,
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
        token::mint = market.meta.long_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market.meta.long_token_mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub long_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = market.meta.short_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market.meta.short_token_mint.as_ref(),
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
        token::mint = market.meta.long_token_mint,
        token::authority = store,
        constraint = check_delegation(claimable_long_token_account_for_user, order.fixed.user)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            market.meta.long_token_mint.as_ref(),
            order.fixed.user.as_ref(),
            &config.claimable_time_key(validated_recent_timestamp(&config, recent_timestamp)?)?,
        ],
        bump,
    )]
    pub claimable_long_token_account_for_user: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = market.meta.short_token_mint,
        token::authority = store,
        constraint = check_delegation(claimable_short_token_account_for_user, order.fixed.user)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            market.meta.short_token_mint.as_ref(),
            order.fixed.user.as_ref(),
            &config.claimable_time_key(validated_recent_timestamp(&config, recent_timestamp)?)?,
        ],
        bump,
    )]
    pub claimable_short_token_account_for_user: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = get_pnl_token(&position, &market)?,
        token::authority = store,
        constraint = check_delegation(claimable_pnl_token_account_for_holding, config.holding()?)?,
        seeds = [
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.key().as_ref(),
            get_pnl_token(&position, &market)?.as_ref(),
            config.holding()?.as_ref(),
            &config.claimable_time_key(validated_recent_timestamp(&config, recent_timestamp)?)?,
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
) -> Result<(bool, Box<TransferOut>)> {
    ctx.accounts.validate_time()?;
    // TODO: validate non-empty order.
    // TODO: validate order trigger price.
    ctx.accounts.pre_execute()?;
    let res = ctx.accounts.execute(ctx.remaining_accounts)?;
    // TODO: validate market state.
    // TODO: emit order executed event.
    Ok(res)
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

    fn execute(
        &mut self,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(bool, Box<TransferOut>)> {
        let mut transfer_out = Box::default();
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

                // Exit must be called to update the external state.
                self.market.exit(&crate::ID)?;
                // CHECK: `exit` has been called above, and `reload` will be called later.
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
                self.process_increase_report(&mut transfer_out, report)?;
                self.order.fixed.params.initial_collateral_delta_amount = 0;
            }
            OrderKind::MarketDecrease | OrderKind::Liquidation => {
                let report = {
                    let Some(position) = self.position.as_mut() else {
                        return err!(DataStoreError::PositionNotProvided);
                    };
                    let params = &self.order.fixed.params;
                    let collateral_withdrawal_amount =
                        params.initial_collateral_delta_amount as u128;
                    let size_delta_usd = params.size_delta_usd;
                    let acceptable_price = params.acceptable_price;

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
                    should_remove = report.should_remove();
                    report
                };
                self.process_decrease_report(&mut transfer_out, remaining_accounts, &report)?;
            }
        }
        Ok((should_remove, transfer_out))
    }

    fn process_increase_report(
        &self,
        ctx: &mut TransferOut,
        report: IncreasePositionReport<u128>,
    ) -> Result<()> {
        msg!("{:?}", report);
        let (long_amount, short_amount) = report.claimable_funding_amounts();
        self.transfer_out_funding_amounts(ctx, long_amount, short_amount)?;
        Ok(())
    }

    fn process_decrease_report(
        &mut self,
        ctx: &mut TransferOut,
        remaining_accounts: &'info [AccountInfo<'info>],
        report: &DecreasePositionReport<u128>,
    ) -> Result<()> {
        msg!("{:?}", report);

        self.process_decrease_swap(ctx, remaining_accounts, report)?;

        // Transfer out funding rebate.
        let (long_amount, short_amount) = report.claimable_funding_amounts();
        self.transfer_out_funding_amounts(ctx, long_amount, short_amount)?;

        self.process_claimable_collateral_for_decrease(ctx, report)?;

        Ok(())
    }

    fn process_decrease_swap(
        &mut self,
        ctx: &mut TransferOut,
        remaining_accounts: &'info [AccountInfo<'info>],
        report: &DecreasePositionReport<u128>,
    ) -> Result<()> {
        require!(
            *report.secondary_output_amount() == 0
                || (report.is_output_token_long() != report.is_secondary_output_token_long()),
            DataStoreError::SameSecondaryTokensNotMerged,
        );

        let (is_output_token_long, output_amount, secondary_output_amount) = (
            report.is_output_token_long(),
            (*report.output_amount())
                .try_into()
                .map_err(|_| DataStoreError::AmountOverflow)?,
            (*report.secondary_output_amount())
                .try_into()
                .map_err(|_| DataStoreError::AmountOverflow)?,
        );

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

        self.transfer_out(ctx, false, output_amount)?;
        self.transfer_out(ctx, true, secondary_amount)?;
        Ok(())
    }

    fn process_claimable_collateral_for_decrease(
        &self,
        ctx: &mut TransferOut,
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
            ctx,
            is_secondary_token_long,
            CollateralReceiver::ClaimableForHolding,
            secondary_amount,
        )?;

        let for_user = report.claimable_collateral_for_user();
        self.transfer_out_collateral(
            ctx,
            is_output_token_long,
            CollateralReceiver::ClaimableForUser,
            (*for_user.output_token_amount())
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            ctx,
            is_secondary_token_long,
            CollateralReceiver::ClaimableForUser,
            (*for_user.secondary_output_token_amount())
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        Ok(())
    }

    fn transfer_out(&self, ctx: &mut TransferOut, is_secondary: bool, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        if is_secondary {
            ctx.final_secondary_output_token = ctx
                .final_secondary_output_token
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            ctx.final_output_token = ctx
                .final_output_token
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        Ok(())
    }

    fn transfer_out_collateral(
        &self,
        ctx: &mut TransferOut,
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
                    ctx.long_token = ctx
                        .long_token
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    ctx.short_token = ctx
                        .short_token
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForHolding => {
                if is_long {
                    ctx.long_token_for_claimable_account_of_holding = ctx
                        .long_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    ctx.short_token_for_claimable_account_of_holding = ctx
                        .short_token_for_claimable_account_of_holding
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
            CollateralReceiver::ClaimableForUser => {
                if is_long {
                    ctx.long_token_for_claimable_account_of_user = ctx
                        .long_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                } else {
                    ctx.short_token_for_claimable_account_of_user = ctx
                        .short_token_for_claimable_account_of_user
                        .checked_add(amount)
                        .ok_or(error!(DataStoreError::AmountOverflow))?;
                }
            }
        }
        Ok(())
    }

    fn transfer_out_funding_amounts(
        &self,
        ctx: &mut TransferOut,
        long_amount: &u128,
        short_amount: &u128,
    ) -> Result<()> {
        self.transfer_out_collateral(
            ctx,
            true,
            CollateralReceiver::Funding,
            (*long_amount)
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        self.transfer_out_collateral(
            ctx,
            false,
            CollateralReceiver::Funding,
            (*short_amount)
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?,
        )?;
        Ok(())
    }
}

fn get_pnl_token(
    position: &Option<AccountLoader<'_, Position>>,
    market: &Market,
) -> Result<Pubkey> {
    let is_long = position
        .as_ref()
        .ok_or(error!(DataStoreError::MissingPosition))?
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
        .ok_or(error!(DataStoreError::NoDelegatedAuthorityIsSet))?;
    Ok(is_matched)
}

fn validated_recent_timestamp(config: &Config, timestamp: i64) -> Result<i64> {
    let recent_time_window = config.recent_time_window()?;
    let expiration_time = timestamp.saturating_add_unsigned(recent_time_window);
    let clock = Clock::get()?;
    if timestamp <= clock.unix_timestamp && clock.unix_timestamp <= expiration_time {
        Ok(timestamp)
    } else {
        err!(DataStoreError::InvalidArgument)
    }
}

enum CollateralReceiver {
    Funding,
    ClaimableForHolding,
    ClaimableForUser,
}
