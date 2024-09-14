use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::{
    states::{withdrawal::WithdrawalV2, Market, NonceBytes, Store},
    CoreError,
};

/// Create Withdrawal Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateWithdrawalParams {
    /// Execution fee in lamports.
    pub execution_fee: u64,
    /// The length of the swap path for long token.
    pub long_token_swap_path_length: u8,
    /// The length of the swap path for short token.
    pub short_token_swap_path_length: u8,
    /// Market token amount to burn.
    pub market_token_amount: u64,
    /// The minimum acceptable final long token amount to receive.
    pub min_long_token_amount: u64,
    /// The minimum acceptable final short token amount to receive.
    pub min_short_token_amount: u64,
}

/// Create Withdrawal Ops.
#[derive(TypedBuilder)]
pub(crate) struct CreateWithdrawalOps<'a, 'info> {
    withdrawal: AccountLoader<'info, WithdrawalV2>,
    market: AccountLoader<'info, Market>,
    store: AccountLoader<'info, Store>,
    owner: &'a AccountInfo<'info>,
    nonce: &'a NonceBytes,
    bump: u8,
    final_long_token: &'a Account<'info, TokenAccount>,
    final_short_token: &'a Account<'info, TokenAccount>,
    market_token: &'a Account<'info, TokenAccount>,
    params: &'a CreateWithdrawalParams,
    swap_paths: &'info [AccountInfo<'info>],
}

impl<'a, 'info> CreateWithdrawalOps<'a, 'info> {
    /// Execute.
    pub(crate) fn execute(self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())?;
        self.validate_params_excluding_swap()?;

        let Self {
            withdrawal,
            market,
            store,
            owner,
            nonce,
            bump,
            final_long_token,
            final_short_token,
            market_token,
            params,
            swap_paths,
        } = self;

        let id = market.load_mut()?.state_mut().next_withdrawal_id()?;

        let mut withdrawal = withdrawal.load_init()?;

        // Initialize header.
        withdrawal
            .header
            .init(id, store.key(), market.key(), owner.key(), *nonce, bump);

        // Initialize time info.
        let clock = Clock::get()?;
        withdrawal.updated_at = clock.unix_timestamp;
        withdrawal.updated_at_slot = clock.slot;

        // Initialize tokens.
        withdrawal.tokens.market_token.init(market_token);
        withdrawal.tokens.final_long_token.init(final_long_token);
        withdrawal.tokens.final_short_token.init(final_short_token);

        // Initialize params.
        withdrawal.params.market_token_amount = params.market_token_amount;
        withdrawal.params.min_long_token_amount = params.min_long_token_amount;
        withdrawal.params.min_short_token_amount = params.min_short_token_amount;
        withdrawal.params.max_execution_lamports = params.execution_fee;

        // Initialize swap paths.
        let market = market.load()?;
        let meta = market.meta();
        withdrawal.swap.validate_and_init(
            &*market,
            params.long_token_swap_path_length,
            params.short_token_swap_path_length,
            swap_paths,
            &store.key(),
            (&meta.long_token_mint, &meta.short_token_mint),
            (&final_long_token.mint, &final_short_token.mint),
        )?;

        Ok(())
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        let params = &self.params;
        require!(params.market_token_amount != 0, CoreError::EmptyWithdrawal);

        require_gte!(
            params.execution_fee,
            WithdrawalV2::MIN_EXECUTION_LAMPORTS,
            CoreError::NotEnoughExecutionFee
        );

        require_gte!(
            self.withdrawal.get_lamports(),
            params.execution_fee,
            CoreError::NotEnoughExecutionFee
        );

        Ok(())
    }
}
