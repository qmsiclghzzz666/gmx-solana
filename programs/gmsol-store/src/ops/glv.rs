use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_interface,
};
use typed_builder::TypedBuilder;

use crate::{
    states::{
        common::action::ActionExt, Glv, GlvDeposit, Market, NonceBytes, Oracle, Store,
        ValidateOracleTime,
    },
    CoreError, CoreResult,
};

/// Create GLV Deposit Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateGlvDepositParams {
    /// Execution fee in lamports
    pub execution_lamports: u64,
    /// The length of the swap path for long token.
    pub long_token_swap_length: u8,
    /// The length of the swap path for short token.
    pub short_token_swap_length: u8,
    /// Initial long token amount to deposit.
    pub initial_long_token_amount: u64,
    /// Initial short otken amount to deposit.
    pub initial_short_token_amount: u64,
    /// Market token amount.
    pub market_token_amount: u64,
    /// The minimum acceptable maount of market tokens to be minted.
    pub min_market_token_amount: u64,
    /// The minimum acceptable amount of glv tokens receive.
    pub min_glv_token_amount: u64,
}

/// Operation for creating GLV deposit.
#[derive(TypedBuilder)]
pub(crate) struct CreateGlvDepositOperation<'a, 'info> {
    glv_deposit: AccountLoader<'info, GlvDeposit>,
    market: AccountLoader<'info, Market>,
    store: AccountLoader<'info, Store>,
    owner: AccountInfo<'info>,
    nonce: &'a NonceBytes,
    bump: u8,
    initial_long_token: Option<&'a Account<'info, TokenAccount>>,
    initial_short_token: Option<&'a Account<'info, TokenAccount>>,
    market_token: &'a Account<'info, TokenAccount>,
    glv_token: &'a InterfaceAccount<'info, token_interface::TokenAccount>,
    params: &'a CreateGlvDepositParams,
    swap_paths: &'info [AccountInfo<'info>],
}

impl<'a, 'info> CreateGlvDepositOperation<'a, 'info> {
    /// Execute.
    ///
    /// # CHECK
    /// - The address of `glv_deposit` must be derived from the given `store`,
    ///   `owner`, `nonce` and `bump`.
    /// - `glv` must be owned by the given `store`.
    /// - `glv` must contain the given `market`.
    /// - All the token accounts must be owned by `glv_deposit`.
    /// - The mint of `market_token` account must be the market token of the
    ///   given `market`.
    /// - The mint of `glv_token` account must be the glv token of the given `glv`.
    ///
    /// # Errors
    /// - `market` must be initialized, enabled and owned by the given `store`.
    /// - `initial_long_token_amount`, `initial_short_token_amount` and `market_token_amount`
    ///   must not all be zero.
    /// - When the token amount is not zero, the corresponding token account must be provided and
    ///   have enough amount of tokens.
    /// - `execution_lamports` must greater than `MIN_EXECUTION_LAMPORTS` and there must be enough lamports
    ///   in the `glv_deposit` account.
    /// - `glv_deposit` must be uninitialized.
    /// - `swap_paths` must be valid.
    pub(crate) fn unchecked_execute(self) -> Result<()> {
        let (long_token, short_token) = self.validate_market_and_get_tokens()?;

        self.validate_params_excluding_swap()?;

        let id = self.market.load_mut()?.state_mut().next_glv_deposit_id()?;

        let mut glv_deposit = self.glv_deposit.load_init()?;

        glv_deposit.header.init(
            id,
            self.store.key(),
            self.market.key(),
            self.owner.key(),
            *self.nonce,
            self.bump,
            self.params.execution_lamports,
        )?;

        // Init tokens and token accounts.
        let primary_token_in = if let Some(account) = self.initial_long_token {
            glv_deposit.tokens.initial_long_token.init(account);
            account.mint
        } else {
            long_token
        };

        let secondary_token_in = if let Some(account) = self.initial_short_token {
            glv_deposit.tokens.initial_short_token.init(account);
            account.mint
        } else {
            short_token
        };
        glv_deposit.tokens.market_token.init(self.market_token);
        glv_deposit
            .tokens
            .glv_token
            .init_with_interface(self.glv_token);

        // Init params.
        glv_deposit.params.initial_long_token_amount = self.params.initial_long_token_amount;
        glv_deposit.params.initial_short_token_amount = self.params.initial_short_token_amount;
        glv_deposit.params.market_token_amount = self.params.market_token_amount;
        glv_deposit.params.min_market_token_amount = self.params.min_market_token_amount;
        glv_deposit.params.min_glv_token_amount = self.params.min_glv_token_amount;

        // Init swap path.
        glv_deposit.swap.validate_and_init(
            &*self.market.load()?,
            self.params.long_token_swap_length,
            self.params.short_token_swap_length,
            self.swap_paths,
            &self.store.key(),
            (&primary_token_in, &secondary_token_in),
            (&long_token, &short_token),
        )?;

        Ok(())
    }

    fn validate_market_and_get_tokens(&self) -> Result<(Pubkey, Pubkey)> {
        let market = self.market.load()?;
        let meta = market.validated_meta(&self.store.key())?;
        Ok((meta.long_token_mint, meta.short_token_mint))
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        let params = self.params;
        require!(
            params.initial_long_token_amount != 0
                || params.initial_short_token_amount != 0
                || params.market_token_amount != 0,
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

        // If the two token accounts are actually the same, then we should check for the sum.
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

        ActionExt::validate_balance(&self.glv_deposit, params.execution_lamports)?;

        Ok(())
    }
}

/// Operation for executing a GLV deposit.
#[derive(TypedBuilder)]
pub(crate) struct ExecuteGlvDepositOperation<'a, 'info> {
    glv_deposit: AccountLoader<'info, GlvDeposit>,
    token_program: AccountInfo<'info>,
    throw_on_execution_error: bool,
    store: AccountLoader<'info, Store>,
    glv: AccountLoader<'info, Glv>,
    market: AccountLoader<'info, Market>,
    market_token_mint: &'a mut Account<'info, Mint>,
    market_token_vault: AccountInfo<'info>,
    markets: &'info [AccountInfo<'info>],
    market_tokens: &'info [AccountInfo<'info>],
    oralce: &'a Oracle,
    remaining_accounts: &'info [AccountInfo<'info>],
}

impl<'a, 'info> ExecuteGlvDepositOperation<'a, 'info> {
    /// Execute.
    ///
    /// # CHECK
    /// - The `glv_deposit` must be owned by the `store`.
    /// - The `glv` must be owned by the `store`, and be the GLV account of the `glv_deposit`.
    /// - The `market_token_mint` must be the market token of the `market`.
    /// - The `market_token_vault` must be the vault of GLV for the `market_token`.
    /// - The lengths of `markets` and `market_tokens` must be the same as the market tokens list of the `glv`.
    /// - The order of `markets` and `market_tokens` must be the same of the market tokens list of the `glv`.
    /// - The required prices of tokens must have been validated and stored in the `oracle`.
    ///
    /// # Errors
    /// - The `market` must be owned by the `store` and be the current market of the `glv_deposit`.
    /// - The swap markets provided by `remaining_accounts` must be valid.
    pub(crate) fn unchecked_execute(self) -> Result<bool> {
        let throw_on_execution_error = self.throw_on_execution_error;
        match self.validate_oracle() {
            Ok(()) => {}
            Err(CoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
                msg!(
                    "GLV Deposit expired at {}",
                    self.oracle_updated_before()
                        .ok()
                        .flatten()
                        .expect("must have an expiration time"),
                );
            }
            Err(err) => {
                return Err(error!(err));
            }
        }
        match self.perform_glv_deposit() {
            Ok(()) => Ok(true),
            Err(err) if !throw_on_execution_error => {
                msg!("Execute GLV deposit error: {}", err);
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    fn validate_oracle(&self) -> CoreResult<()> {
        self.oralce.validate_time(self)
    }

    fn validate_market_and_glv_deposit(&self) -> Result<()> {
        let market = self.market.load()?;
        market.validate(&self.store.key())?;

        self.glv_deposit
            .load()?
            .validate_for_execution(&self.market_token_mint.to_account_info(), &market)?;

        Ok(())
    }

    fn perform_glv_deposit(&self) -> Result<()> {
        self.validate_market_and_glv_deposit()?;
        todo!()
    }
}

impl<'a, 'info> ValidateOracleTime for ExecuteGlvDepositOperation<'a, 'info> {
    fn oracle_updated_after(&self) -> CoreResult<Option<i64>> {
        Ok(Some(
            self.glv_deposit
                .load()
                .map_err(|_| CoreError::LoadAccountError)?
                .header
                .updated_at,
        ))
    }

    fn oracle_updated_before(&self) -> CoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| CoreError::LoadAccountError)?
            .request_expiration_at(
                self.glv_deposit
                    .load()
                    .map_err(|_| CoreError::LoadAccountError)?
                    .header
                    .updated_at,
            )?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> CoreResult<Option<u64>> {
        Ok(Some(
            self.glv_deposit
                .load()
                .map_err(|_| CoreError::LoadAccountError)?
                .header
                .updated_at_slot,
        ))
    }
}
