use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use gmsol_model::{LiquidityMarketMutExt, PositionImpactMarketMutExt};

use crate::{
    constants,
    ops::{
        execution_fee::PayExecutionFeeOps, market::MarketTransferOut,
        withdrawal::ExecuteWithdrawalOp,
    },
    states::{
        common::action::ActionSigner,
        ops::ValidateMarketBalances,
        revertible::{
            swap_market::{SwapDirection, SwapMarkets},
            Revertible, RevertibleLiquidityMarket,
        },
        withdrawal::WithdrawalV2,
        HasMarketMeta, Market, Oracle, PriceProvider, Seed, Store, TokenMapHeader, TokenMapLoader,
        ValidateOracleTime, Withdrawal,
    },
    utils::internal,
    CoreError, ModelError, StoreError, StoreResult,
};

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        constraint = withdrawal.fixed.store == store.key(),
        constraint = withdrawal.fixed.market == market.key(),
        constraint = withdrawal.fixed.tokens.market_token == market_token_mint.key(),
        constraint = withdrawal.fixed.receivers.final_long_token_receiver == final_long_token_receiver.key(),
        constraint = withdrawal.fixed.receivers.final_short_token_receiver == final_short_token_receiver.key(),
        seeds = [
            Withdrawal::SEED,
            store.key().as_ref(),
            withdrawal.fixed.user.as_ref(),
            &withdrawal.fixed.nonce,
        ],
        bump = withdrawal.fixed.bump,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = market_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_withdrawal_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub market_token_withdrawal_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = final_long_token_receiver.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_long_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_long_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = final_short_token_receiver.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_short_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_short_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub final_long_token_receiver: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub final_short_token_receiver: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

/// Execute a withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    throw_on_execution_error: bool,
) -> Result<(u64, u64)> {
    match ctx.accounts.validate_oracle() {
        Ok(()) => {}
        Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
            msg!(
                "Withdrawal expired at {}",
                ctx.accounts
                    .oracle_updated_before()
                    .ok()
                    .flatten()
                    .expect("must have an expiration time"),
            );
            return Ok((0, 0));
        }
        Err(err) => {
            return Err(error!(err));
        }
    }
    match ctx.accounts.execute(ctx.remaining_accounts) {
        Ok(res) => Ok(res),
        Err(err) if !throw_on_execution_error => {
            // TODO: catch and throw missing oracle price error.
            msg!("Execute withdrawal error: {}", err);
            Ok((0, 0))
        }
        Err(err) => Err(err),
    }
}

impl<'info> internal::Authentication<'info> for ExecuteWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ValidateOracleTime for ExecuteWithdrawal<'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(self.withdrawal.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(self.withdrawal.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(self.withdrawal.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<(u64, u64)> {
        self.validate_market()?;

        // Prepare the execution context.
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket::new(
            &self.market,
            &mut self.market_token_mint,
            self.token_program.to_account_info(),
            &self.store,
        )?
        .enable_burn(self.market_token_withdrawal_vault.to_account_info());
        let loaders = self
            .withdrawal
            .dynamic
            .swap
            .unpack_markets_for_swap(&current_market_token, remaining_accounts)?;
        let mut swap_markets =
            SwapMarkets::new(&self.store.key(), &loaders, Some(&current_market_token))?;

        // Distribute position impact.
        {
            let report = market
                .distribute_position_impact()
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Withdrawal] pre-execute: {:?}", report);
        }

        // Perform the withdrawal.
        let (long_amount, short_amount) = {
            let prices = self.oracle.market_prices(&market)?;
            let report = market
                .withdraw(
                    self.withdrawal.fixed.tokens.market_token_amount.into(),
                    prices,
                )
                .and_then(|w| w.execute())
                .map_err(ModelError::from)?;
            let (long_amount, short_amount) = (
                (*report.long_token_output())
                    .try_into()
                    .map_err(|_| StoreError::AmountOverflow)?,
                (*report.short_token_output())
                    .try_into()
                    .map_err(|_| StoreError::AmountOverflow)?,
            );
            // Validate current market.
            market.validate_market_balances(long_amount, short_amount)?;
            msg!("[Withdrawal] executed: {:?}", report);
            (long_amount, short_amount)
        };

        // Perform the swap.
        let (final_long_amount, final_short_amount) = {
            let meta = *market.market_meta();
            swap_markets.revertible_swap(
                SwapDirection::From(&mut market),
                &self.oracle,
                &self.withdrawal.dynamic.swap,
                (
                    self.withdrawal.fixed.tokens.final_long_token,
                    self.withdrawal.fixed.tokens.final_short_token,
                ),
                (Some(meta.long_token_mint), Some(meta.short_token_mint)),
                (long_amount, short_amount),
            )?
        };

        self.withdrawal
            .validate_output_amounts(final_long_amount, final_short_amount)?;

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        self.withdrawal.fixed.tokens.market_token_amount = 0;

        Ok((final_long_amount, final_short_amount))
    }
}

/// The accounts deifinition for the `execute_withdrawal` instruction.
#[derive(Accounts)]
pub struct ExecuteWithdrawalV2<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// Token map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Price provider.
    pub price_provider: Interface<'info, PriceProvider>,
    /// Oracle buffer to use.
    #[account(has_one = store)]
    pub oracle: Box<Account<'info, Oracle>>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The withdrawal to execute.
    #[account(
        mut,
        constraint = withdrawal.load()?.header.market == market.key() @ CoreError::MarketMismatched,
        constraint = withdrawal.load()?.header.store == store.key() @ CoreError::StoreMismatched,
        constraint = withdrawal.load()?.tokens.market_token_account() == market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = withdrawal.load()?.tokens.final_long_token_account() == final_long_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = withdrawal.load()?.tokens.final_short_token_account() == final_short_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
    )]
    pub withdrawal: AccountLoader<'info, WithdrawalV2>,
    /// Market token.
    #[account(
        constraint = withdrawal.load()?.tokens.market_token() == market_token.key() @ CoreError::MarketTokenMintMismatched
    )]
    pub market_token: Box<Account<'info, Mint>>,
    /// Final long token.
    #[account(constraint = withdrawal.load()?.tokens.final_long_token() == final_long_token.key() @ CoreError::TokenMintMismatched)]
    pub final_long_token: Box<Account<'info, Mint>>,
    /// Final short token.
    #[account(constraint = withdrawal.load()?.tokens.final_long_token() == final_long_token.key() @ CoreError::TokenMintMismatched)]
    pub final_short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving market tokens to burn.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = withdrawal,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final long tokens.
    #[account(
        mut,
        associated_token::mint = final_long_token,
        associated_token::authority = withdrawal,
    )]
    pub final_long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving withdrawed final short tokens.
    #[account(
        mut,
        associated_token::mint = final_short_token,
        associated_token::authority = withdrawal,
    )]
    pub final_short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// Market token vault.
    #[account(
        mut,
        token::mint = market_token,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub market_token_vault: Box<Account<'info, TokenAccount>>,
    /// Final long token vault.
    #[account(
        mut,
        token::mint = final_long_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_long_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_long_token_vault: Box<Account<'info, TokenAccount>>,
    /// Final short token vault.
    #[account(
        mut,
        token::mint = final_short_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_short_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_short_token_vault: Box<Account<'info, TokenAccount>>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// CHECK only ORDER_KEEPER can invoke this instruction.
pub(crate) fn unchecked_execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawalV2<'info>>,
    execution_fee: u64,
    throw_on_execution_error: bool,
) -> Result<()> {
    let accounts = ctx.accounts;
    let remaining_accounts = ctx.remaining_accounts;
    let signer = accounts.withdrawal.load()?.signer();

    accounts.transfer_market_tokens_in(&signer)?;

    let executed = accounts.perform_execution(remaining_accounts, throw_on_execution_error)?;

    match executed {
        Some((final_long_token_amount, final_short_token_amount)) => {
            accounts.withdrawal.load_mut()?.header.completed()?;
            accounts.transfer_tokens_out(
                remaining_accounts,
                final_long_token_amount,
                final_short_token_amount,
            )?;
        }
        None => {
            accounts.withdrawal.load_mut()?.header.cancelled()?;
            accounts.transfer_market_tokens_out()?;
        }
    }

    // Is must be placed at the end to be executed correctly.
    accounts.pay_execution_fee(execution_fee)?;

    Ok(())
}

impl<'info> internal::Authentication<'info> for ExecuteWithdrawalV2<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ExecuteWithdrawalV2<'info> {
    fn perform_execution(
        &mut self,
        remaining_accounts: &'info [AccountInfo<'info>],
        throw_on_execution_error: bool,
    ) -> Result<Option<(u64, u64)>> {
        // FIXME: We only need the tokens here, the feeds are not necessary.
        let feeds = self
            .withdrawal
            .load()?
            .swap()
            .to_feeds(&self.token_map.load_token_map()?)?;

        let op = ExecuteWithdrawalOp::builder()
            .store(&self.store)
            .market(&self.market)
            .withdrawal(&self.withdrawal)
            .market_token_mint(&mut self.market_token)
            .market_token_vault(self.market_token_vault.to_account_info())
            .token_program(self.token_program.to_account_info())
            .throw_on_execution_error(throw_on_execution_error);

        let executed = self.oracle.with_prices(
            &self.store,
            &self.price_provider,
            &self.token_map,
            &feeds.tokens,
            remaining_accounts,
            |oracle, remaining_accounts| {
                op.oracle(oracle)
                    .remaining_accounts(remaining_accounts)
                    .build()
                    .execute()
            },
        )?;

        Ok(executed)
    }

    fn transfer_market_tokens_in(&self, signer: &ActionSigner) -> Result<()> {
        let seeds = signer.as_seeds();

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.market_token_escrow.to_account_info(),
                    mint: self.market_token.to_account_info(),
                    to: self.market_token_vault.to_account_info(),
                    authority: self.withdrawal.to_account_info(),
                },
            )
            .with_signer(&[&seeds]),
            self.withdrawal.load()?.params.market_token_amount,
            self.market_token.decimals,
        )?;

        Ok(())
    }

    fn transfer_market_tokens_out(&self) -> Result<()> {
        use crate::internal::TransferUtils;

        let amount = self.withdrawal.load()?.params.market_token_amount;
        TransferUtils::new(self.token_program.to_account_info(), &self.store, None).transfer_out(
            self.market_token_vault.to_account_info(),
            self.market_token_escrow.to_account_info(),
            amount,
        )?;

        Ok(())
    }

    fn transfer_tokens_out(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
        final_long_token_amount: u64,
        final_short_token_amount: u64,
    ) -> Result<()> {
        let builder = MarketTransferOut::builder()
            .store(&self.store)
            .token_program(self.token_program.to_account_info());
        let store = &self.store.key();

        if final_long_token_amount != 0 {
            let market = self
                .withdrawal
                .load()?
                .swap
                .find_and_unpack_last_market(store, true, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = &self.final_long_token_vault;
            let escrow = &self.final_long_token_escrow;
            builder
                .clone()
                .market(&market)
                .to(escrow.to_account_info())
                .vault(vault)
                .amount(final_long_token_amount)
                .build()
                .execute()?;
        }

        if final_short_token_amount != 0 {
            let market = self
                .withdrawal
                .load()?
                .swap
                .find_and_unpack_last_market(store, true, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = &self.final_short_token_vault;
            let escrow = &self.final_short_token_escrow;
            builder
                .market(&market)
                .to(escrow.to_account_info())
                .vault(vault)
                .amount(final_short_token_amount)
                .build()
                .execute()?;
        }
        Ok(())
    }

    fn pay_execution_fee(&self, execution_fee: u64) -> Result<()> {
        let execution_lamports =
            execution_fee.min(self.withdrawal.load()?.params.max_execution_lamports);
        PayExecutionFeeOps::builder()
            .payer(self.withdrawal.to_account_info())
            .receiver(self.authority.to_account_info())
            .execution_lamports(execution_lamports)
            .build()
            .execute()?;
        Ok(())
    }
}
