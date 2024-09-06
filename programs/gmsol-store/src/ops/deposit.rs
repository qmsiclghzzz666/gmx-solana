use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::{
    states::{DepositV2, Market, NonceBytes, Store},
    CoreError,
};

/// Create Deposit Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateDepositParams {
    pub execution_fee: u64,
    pub long_token_swap_length: u8,
    pub short_token_swap_length: u8,
    pub initial_long_token_decimals: u8,
    pub initial_long_token_amount: u64,
    pub initial_short_token_decimals: u8,
    pub initial_short_token_amount: u64,
    pub min_market_token: u64,
}

/// Create Deposit Ops.
#[derive(TypedBuilder)]
pub(crate) struct CreateDepositOps<'a, 'info> {
    deposit: AccountLoader<'info, DepositV2>,
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

        deposit.id = id;
        deposit.store = store.key();
        deposit.market = market.key();
        deposit.owner = owner.key();
        deposit.nonce = *nonce;
        deposit.bump = bump;

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
        deposit.params.max_execution_lamports = params.execution_fee;

        deposit.swap.validate_and_init(
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

        require_gte!(
            params.execution_fee,
            DepositV2::MIN_EXECUTION_LAMPORTS,
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
