use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_model::{LiquidityMarketMutExt, PositionImpactMarketMutExt};

use crate::{
    constants,
    ops::{
        deposit::ExecuteDepositOps,
        execution_fee::PayExecutionFeeOps,
        market::{MarketTransferIn, MarketTransferOut},
    },
    states::{
        common::action::ActionSigner,
        ops::ValidateMarketBalances,
        revertible::{
            swap_market::{SwapDirection, SwapMarkets},
            Revertible, RevertibleLiquidityMarket,
        },
        Deposit, DepositV2, HasMarketMeta, Market, Oracle, PriceProvider, Seed, Store,
        TokenMapHeader, TokenMapLoader, ValidateOracleTime,
    },
    utils::internal,
    CoreError, ModelError, StoreError, StoreResult,
};

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    #[account(
        // The `mut` flag must be present, since we are mutating the deposit.
        // It may not throw any errors sometimes if we forget to mark the account as mutable,
        // so be careful.
        mut,
        constraint = deposit.fixed.store == store.key(),
        constraint = deposit.fixed.receivers.receiver == receiver.key(),
        constraint = deposit.fixed.tokens.market_token == market_token_mint.key(),
        constraint = deposit.fixed.market == market.key(),
        seeds = [
            Deposit::SEED,
            store.key().as_ref(),
            deposit.fixed.senders.user.key().as_ref(),
            &deposit.fixed.nonce,
        ],
        bump = deposit.fixed.bump,
    )]
    pub deposit: Account<'info, Deposit>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    throw_on_execution_error: bool,
) -> Result<bool> {
    match ctx.accounts.validate_oracle() {
        Ok(()) => {}
        Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
            msg!(
                "Deposit expired at {}",
                ctx.accounts
                    .oracle_updated_before()
                    .ok()
                    .flatten()
                    .expect("must have an expiration time"),
            );
            return Ok(false);
        }
        Err(err) => {
            return Err(error!(err));
        }
    }
    match ctx.accounts.execute(ctx.remaining_accounts) {
        Ok(()) => Ok(true),
        Err(err) if !throw_on_execution_error => {
            // TODO: catch and throw missing oracle price error.
            msg!("Execute deposit error: {}", err);
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

impl<'info> internal::Authentication<'info> for ExecuteDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ValidateOracleTime for ExecuteDeposit<'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(self.deposit.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(self.deposit.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(self.deposit.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteDeposit<'info> {
    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        self.validate_market()?;

        // TODO: validate first deposit.

        // Prepare the execution context.
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket::new(
            &self.market,
            &mut self.market_token_mint,
            self.token_program.to_account_info(),
            &self.store,
        )?
        .enable_mint(self.receiver.to_account_info());
        let loaders = self
            .deposit
            .dynamic
            .swap_params
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
            msg!("[Deposit] pre-execute: {:?}", report);
        }

        // Swap tokens into the target market.
        let (long_token_amount, short_token_amount) = {
            let meta = market.market_meta();
            let expected_token_outs = (meta.long_token_mint, meta.short_token_mint);
            swap_markets.revertible_swap(
                SwapDirection::Into(&mut market),
                &self.oracle,
                &self.deposit.dynamic.swap_params,
                expected_token_outs,
                (
                    self.deposit.fixed.tokens.initial_long_token,
                    self.deposit.fixed.tokens.initial_short_token,
                ),
                (
                    self.deposit.fixed.tokens.params.initial_long_token_amount,
                    self.deposit.fixed.tokens.params.initial_short_token_amount,
                ),
            )?
        };

        // Perform the deposit.
        {
            let prices = self.oracle.market_prices(&market)?;
            let report = market
                .deposit(long_token_amount.into(), short_token_amount.into(), prices)
                .and_then(|d| d.execute())
                .map_err(ModelError::from)?;
            market.validate_market_balances(0, 0)?;

            self.deposit.validate_min_market_tokens(
                (*report.minted())
                    .try_into()
                    .map_err(|_| error!(StoreError::AmountOverflow))?,
            )?;

            msg!("[Deposit] executed: {:?}", report);
        }

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        // Set amounts to zero to make sure it can be removed without transferring out any tokens.
        self.deposit.fixed.tokens.params.initial_long_token_amount = 0;
        self.deposit.fixed.tokens.params.initial_short_token_amount = 0;
        Ok(())
    }
}

/// The accounts definition for `execute_deposit` instruction.
#[derive(Accounts)]
pub struct ExecuteDepositV2<'info> {
    /// Authority.
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Token Map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Price Provider.
    pub price_provider: Interface<'info, PriceProvider>,
    /// Oracle buffer to use.
    #[account(has_one = store)]
    pub oracle: Box<Account<'info, Oracle>>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The deposit to execute.
    #[account(
        mut,
        has_one = store,
        has_one = market,
        constraint = deposit.load()?.tokens.market_token.account().expect("must exist") == market_token_escrow.key() @ CoreError::MarketTokenAccountMismatched,
        constraint = deposit.load()?.tokens.initial_long_token.account() == initial_long_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = deposit.load()?.tokens.initial_short_token.account() == initial_short_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        seeds = [DepositV2::SEED, store.key().as_ref(), deposit.load()?.owner.as_ref(), &deposit.load()?.nonce],
        bump = deposit.load()?.bump,
    )]
    pub deposit: AccountLoader<'info, DepositV2>,
    /// Market token mint.
    #[account(mut, constraint = market.load()?.meta().market_token_mint == market_token.key() @ CoreError::MarketTokenMintMismatched)]
    pub market_token: Box<Account<'info, Mint>>,
    /// Initial long token.
    #[account(
        constraint = deposit.load()?.tokens.initial_long_token.token().map(|token| initial_long_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_long_token: Option<Box<Account<'info, Mint>>>,
    /// Initial short token.
    #[account(
        constraint = deposit.load()?.tokens.initial_short_token.token().map(|token| initial_short_token.key() == token).unwrap_or(true) @ CoreError::TokenMintMismatched
    )]
    pub initial_short_token: Option<Box<Account<'info, Mint>>>,
    /// The escrow account for receving market tokens.
    #[account(
        mut,
        associated_token::mint = market_token,
        associated_token::authority = deposit,
    )]
    pub market_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receiving initial long token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_long_token,
        associated_token::authority = deposit,
    )]
    pub initial_long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for receiving initial short token for deposit.
    #[account(
        mut,
        associated_token::mint = initial_short_token,
        associated_token::authority = deposit,
    )]
    pub initial_short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// Initial long token vault.
    #[account(
        mut,
        token::mint = initial_long_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_long_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub initial_long_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    /// Initial short token vault.
    #[account(
        mut,
        token::mint = initial_short_token,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_short_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub initial_short_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// CHECK: only ORDER_KEEPER can invoke this instruction.
#[inline(never)]
pub(crate) fn unchecked_execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDepositV2<'info>>,
    execution_fee: u64,
    throw_on_execution_error: bool,
) -> Result<()> {
    let accounts = ctx.accounts;
    let remaining_accounts = ctx.remaining_accounts;

    let signer = accounts.deposit.load()?.signer();

    accounts.transfer_tokens_in(&signer, remaining_accounts)?;

    let executed = accounts.perform_execution(remaining_accounts, throw_on_execution_error)?;

    if executed {
        accounts.deposit.load_mut()?.completed()?;
    } else {
        accounts.deposit.load_mut()?.cancelled()?;
        accounts.transfer_tokens_out(remaining_accounts)?;
    }

    // It must be placed at the end to be executed correctly.
    accounts.pay_execution_fee(execution_fee)?;

    Ok(())
}

impl<'info> internal::Authentication<'info> for ExecuteDepositV2<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ExecuteDepositV2<'info> {
    #[inline(never)]
    fn pay_execution_fee(&self, execution_fee: u64) -> Result<()> {
        let execution_lamports =
            execution_fee.min(self.deposit.load()?.params.max_execution_lamports);
        PayExecutionFeeOps::builder()
            .payer(self.deposit.to_account_info())
            .receiver(self.authority.to_account_info())
            .execution_lamports(execution_lamports)
            .build()
            .execute()?;
        Ok(())
    }

    #[inline(never)]
    fn transfer_tokens_in(
        &self,
        signer: &ActionSigner,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        let seeds = signer.as_seeds();

        let builder = MarketTransferIn::builder()
            .store(&self.store)
            .from_authority(self.deposit.to_account_info())
            .token_program(self.token_program.to_account_info())
            .signer_seeds(&seeds);

        let store = &self.store.key();

        if let Some(escrow) = self.initial_long_token_escrow.as_ref() {
            let market = self
                .deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, true, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = self
                .initial_long_token_vault
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .from(escrow.to_account_info())
                .vault(vault)
                .amount(self.deposit.load()?.params.initial_long_token_amount)
                .build()
                .execute()?;
        }

        if let Some(escrow) = self.initial_short_token_escrow.as_ref() {
            let market = self
                .deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, false, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = self
                .initial_short_token_vault
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .from(escrow.to_account_info())
                .vault(vault)
                .amount(self.deposit.load()?.params.initial_short_token_amount)
                .build()
                .execute()?;
        }

        Ok(())
    }

    #[inline(never)]
    fn transfer_tokens_out(&self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        let builder = MarketTransferOut::builder()
            .store(&self.store)
            .token_program(self.token_program.to_account_info());

        let store = &self.store.key();

        if let Some(escrow) = self.initial_long_token_escrow.as_ref() {
            let market = self
                .deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, true, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = self
                .initial_long_token_vault
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .to(escrow.to_account_info())
                .vault(vault)
                .amount(self.deposit.load()?.params.initial_long_token_amount)
                .build()
                .execute()?;
        }

        if let Some(escrow) = self.initial_short_token_escrow.as_ref() {
            let market = self
                .deposit
                .load()?
                .swap
                .find_and_unpack_first_market(store, false, remaining_accounts)?
                .unwrap_or(self.market.clone());
            let vault = self
                .initial_short_token_vault
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            builder
                .clone()
                .market(&market)
                .to(escrow.to_account_info())
                .vault(vault)
                .amount(self.deposit.load()?.params.initial_short_token_amount)
                .build()
                .execute()?;
        }

        Ok(())
    }

    #[inline(never)]
    fn perform_execution(
        &mut self,
        remaining_accounts: &'info [AccountInfo<'info>],
        throw_on_execution_error: bool,
    ) -> Result<bool> {
        // FIXME: We only need the tokens here, the feeds are not necessary.
        let feeds = self
            .deposit
            .load()?
            .swap()
            .to_feeds(&self.token_map.load_token_map()?)?;
        let ops = ExecuteDepositOps::builder()
            .store(&self.store)
            .market(&self.market)
            .deposit(&self.deposit)
            .market_token_mint(&mut self.market_token)
            .market_token_receiver(self.market_token_escrow.to_account_info())
            .token_program(self.token_program.to_account_info())
            .throw_on_execution_error(throw_on_execution_error);

        let executed = self.oracle.with_prices(
            &self.store,
            &self.price_provider,
            &self.token_map,
            &feeds.tokens,
            remaining_accounts,
            |oracle, remaining_accounts| {
                ops.oracle(oracle)
                    .remaining_accounts(remaining_accounts)
                    .build()
                    .execute()
            },
        )?;

        Ok(executed)
    }
}
