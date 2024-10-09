use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use typed_builder::TypedBuilder;

use crate::{
    states::{
        common::action::Action, Deposit, Market, NonceBytes, Oracle, Store, ValidateOracleTime,
    },
    CoreError, StoreError, StoreResult,
};

/// Create Deposit Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateDepositParams {
    /// Execution fee in lamports
    pub execution_fee: u64,
    /// The length of the swap path for long token.
    pub long_token_swap_length: u8,
    /// The length of the swap path for short token.
    pub short_token_swap_length: u8,
    /// Initial long token amount to deposit.
    pub initial_long_token_amount: u64,
    /// Initial short otken amount to deposit.
    pub initial_short_token_amount: u64,
    /// The minimum acceptable market token to receive.
    pub min_market_token: u64,
}

/// Create Deposit Ops.
#[derive(TypedBuilder)]
pub(crate) struct CreateDepositOps<'a, 'info> {
    deposit: AccountLoader<'info, Deposit>,
    market: AccountLoader<'info, Market>,
    store: AccountLoader<'info, Store>,
    owner: &'a AccountInfo<'info>,
    nonce: &'a NonceBytes,
    bump: u8,
    #[builder(default)]
    initial_long_token: Option<&'a Account<'info, TokenAccount>>,
    #[builder(default)]
    initial_short_token: Option<&'a Account<'info, TokenAccount>>,
    market_token: &'a Account<'info, TokenAccount>,
    params: &'a CreateDepositParams,
    swap_paths: &'info [AccountInfo<'info>],
}

impl<'a, 'info> CreateDepositOps<'a, 'info> {
    /// Execute.
    pub(crate) fn execute(self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())?;
        self.validate_params_excluding_swap()?;

        let Self {
            bump,
            deposit,
            market,
            store,
            owner,
            nonce,
            initial_long_token,
            initial_short_token,
            market_token,
            params,
            swap_paths,
        } = self;

        let id = market.load_mut()?.state_mut().next_deposit_id()?;

        let mut deposit = deposit.load_init()?;

        deposit.header.init(
            id,
            store.key(),
            market.key(),
            owner.key(),
            *nonce,
            bump,
            params.execution_fee,
        )?;

        let (long_token, short_token) = {
            let market = market.load()?;
            let meta = market.meta();
            (meta.long_token_mint, meta.short_token_mint)
        };

        let primary_token_in = if let Some(account) = initial_long_token {
            deposit.tokens.initial_long_token.init(account);
            account.mint
        } else {
            long_token
        };

        let secondary_token_in = if let Some(account) = initial_short_token {
            deposit.tokens.initial_short_token.init(account);
            account.mint
        } else {
            short_token
        };

        deposit.tokens.market_token.init(market_token);

        deposit.params.initial_long_token_amount = params.initial_long_token_amount;
        deposit.params.initial_short_token_amount = params.initial_short_token_amount;
        deposit.params.min_market_token_amount = params.min_market_token;

        deposit.swap.validate_and_init(
            &*market.load()?,
            params.long_token_swap_length,
            params.short_token_swap_length,
            swap_paths,
            &store.key(),
            (&primary_token_in, &secondary_token_in),
            (&long_token, &short_token),
        )?;

        Ok(())
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        let params = &self.params;
        require!(
            params.initial_long_token_amount != 0 || params.initial_short_token_amount != 0,
            CoreError::EmptyDeposit
        );

        if params.initial_long_token_amount != 0 {
            let Some(account) = self.initial_long_token.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            require_gte!(
                account.amount,
                params.initial_long_token_amount,
                CoreError::NotEnoughTokenAmount
            );
        }

        if params.initial_short_token_amount != 0 {
            let Some(account) = self.initial_short_token.as_ref() else {
                return err!(CoreError::TokenAccountNotProvided);
            };
            require_gte!(
                account.amount,
                params.initial_short_token_amount,
                CoreError::NotEnoughTokenAmount
            );
        }

        // If the two token accounts are actually the same, then we should check the total sum.
        let same_initial_token_amount = self.initial_long_token.as_ref().and_then(|long| {
            self.initial_short_token
                .as_ref()
                .and_then(|short| (long.key() == short.key()).then(|| long.amount))
        });
        if let Some(amount) = same_initial_token_amount {
            let total_amount = params
                .initial_long_token_amount
                .checked_add(params.initial_short_token_amount)
                .ok_or(error!(CoreError::TokenAmountExceedsLimit))?;
            require_gte!(amount, total_amount, CoreError::NotEnoughTokenAmount);
        }

        require_gte!(
            params.execution_fee,
            Deposit::MIN_EXECUTION_LAMPORTS,
            CoreError::NotEnoughExecutionFee
        );

        require_gte!(
            self.deposit.get_lamports(),
            params.execution_fee,
            CoreError::NotEnoughExecutionFee
        );

        Ok(())
    }
}

/// Execute Deposit Ops.
#[derive(TypedBuilder)]
pub(crate) struct ExecuteDepositOps<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    market_token_mint: &'a mut Account<'info, Mint>,
    market_token_receiver: AccountInfo<'info>,
    deposit: &'a AccountLoader<'info, Deposit>,
    oracle: &'a Oracle,
    remaining_accounts: &'info [AccountInfo<'info>],
    throw_on_execution_error: bool,
    token_program: AccountInfo<'info>,
}

impl<'a, 'info> ExecuteDepositOps<'a, 'info> {
    pub(crate) fn execute(self) -> Result<bool> {
        let throw_on_execution_error = self.throw_on_execution_error;
        match self.validate_oracle() {
            Ok(()) => {}
            Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
                msg!(
                    "Deposit expired at {}",
                    self.oracle_updated_before()
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
        match self.do_execute() {
            Ok(()) => Ok(true),
            Err(err) if !throw_on_execution_error => {
                // TODO: catch and throw missing oracle price error.
                msg!("Execute deposit error: {}", err);
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    #[inline(never)]
    fn do_execute(self) -> Result<()> {
        use crate::{
            states::{
                revertible::{
                    swap_market::{SwapDirection, SwapMarkets},
                    Revertible, RevertibleLiquidityMarket,
                },
                HasMarketMeta, ValidateMarketBalances,
            },
            ModelError,
        };
        use gmsol_model::{LiquidityMarketMutExt, PositionImpactMarketMutExt};

        self.validate_market()?;

        // TODO: validate first deposit.

        // Prepare the execution context.
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket::new(
            self.market,
            self.market_token_mint,
            self.token_program.to_account_info(),
            self.store,
        )?
        .enable_mint(self.market_token_receiver.to_account_info());
        let loaders = self
            .deposit
            .load()?
            .swap
            .unpack_markets_for_swap(&current_market_token, self.remaining_accounts)?;
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
                self.oracle,
                &(&self.deposit.load()?.swap).into(),
                expected_token_outs,
                (
                    self.deposit.load()?.tokens.initial_long_token.token(),
                    self.deposit.load()?.tokens.initial_short_token.token(),
                ),
                (
                    self.deposit.load()?.params.initial_long_token_amount,
                    self.deposit.load()?.params.initial_short_token_amount,
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

            let minted: u64 = (*report.minted())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?;

            require_gte!(
                minted,
                self.deposit.load()?.params.min_market_token_amount,
                CoreError::InsufficientOutputAmount
            );

            msg!("[Deposit] executed: {:?}", report);
        }

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        Ok(())
    }
}

impl<'a, 'info> ValidateOracleTime for ExecuteDepositOps<'a, 'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(
            self.deposit
                .load()
                .map_err(|_| StoreError::LoadAccountError)?
                .header()
                .updated_at,
        ))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(
                self.deposit
                    .load()
                    .map_err(|_| StoreError::LoadAccountError)?
                    .header()
                    .updated_at,
            )?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(
            self.deposit
                .load()
                .map_err(|_| StoreError::LoadAccountError)?
                .header()
                .updated_at_slot,
        ))
    }
}
