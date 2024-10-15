use anchor_lang::prelude::*;
use anchor_spl::{token::TokenAccount, token_interface};
use typed_builder::TypedBuilder;

use crate::{
    states::{common::action::ActionExt, GlvDeposit, Market, NonceBytes, Store},
    CoreError,
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
