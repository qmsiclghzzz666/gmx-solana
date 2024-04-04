use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmx_core::MarketExt;

use crate::{
    states::{DataStore, Deposit, Market, MarketMeta, Oracle, Roles},
    utils::internal,
    DataStoreError, GmxCoreError, ID,
};

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_order_keeper: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        constraint = deposit.fixed.receivers.receiver == receiver.key(),
        constraint = deposit.fixed.tokens.market_token == market_token_mint.key(),
        constraint = deposit.fixed.market == market.key(),
    )]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
) -> Result<()> {
    ctx.accounts.execute(ctx.remaining_accounts)
}

impl<'info> internal::Authentication<'info> for ExecuteDeposit<'info> {
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

impl<'info> ExecuteDeposit<'info> {
    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        let meta = self.market.meta.clone();
        let (long_amount, short_amount) = self.perform_swaps(&meta, remaining_accounts)?;
        msg!("{}, {}", long_amount, short_amount);
        self.perform_deposit(&meta, long_amount, short_amount)?;
        Ok(())
    }

    fn perform_swaps(
        &mut self,
        meta: &MarketMeta,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(u64, u64)> {
        let swap_params = &self.deposit.dynamic.swap_params;
        let long_len = swap_params.long_token_swap_path.len();
        let total_len =
            swap_params.long_token_swap_path.len() + swap_params.short_token_swap_path.len();

        // Expecting the `remaining_accounts` are of the of the following form:
        // [...long_path_markets, ...short_path_markets, ...long_path_mints, ...short_path_mints]
        require_gte!(
            remaining_accounts.len(),
            total_len * 2,
            ErrorCode::AccountNotEnoughKeys
        );

        // Markets.
        let long_swap_path = &remaining_accounts[0..long_len];
        let short_swap_path = &remaining_accounts[long_len..total_len];

        // Mints.
        let remaining_accounts = &remaining_accounts[total_len..];
        let long_swap_path_mints = &remaining_accounts[0..long_len];
        let short_swap_path_mints = &remaining_accounts[long_len..total_len];

        let long_amount = self.perform_swap(meta, true, long_swap_path, long_swap_path_mints)?;
        let short_amount =
            self.perform_swap(meta, false, short_swap_path, short_swap_path_mints)?;
        Ok((long_amount, short_amount))
    }

    fn perform_swap(
        &mut self,
        meta: &MarketMeta,
        is_long: bool,
        markets: &'info [AccountInfo<'info>],
        mints: &'info [AccountInfo<'info>],
    ) -> Result<u64> {
        let (token, final_token, mut amount, expected_mints) = if is_long {
            (
                self.deposit.fixed.tokens.initial_long_token,
                meta.long_token_mint,
                self.deposit.fixed.tokens.params.initial_long_token_amount,
                &self.deposit.dynamic.swap_params.long_token_swap_path,
            )
        } else {
            (
                self.deposit.fixed.tokens.initial_short_token,
                meta.short_token_mint,
                self.deposit.fixed.tokens.params.initial_short_token_amount,
                &self.deposit.dynamic.swap_params.short_token_swap_path,
            )
        };

        let Some(mut token_in) = token else {
            return Ok(0);
        };

        let mut flags = BTreeSet::default();
        for (idx, market) in markets.iter().enumerate() {
            require!(flags.insert(market.key), DataStoreError::InvalidSwapPath);
            let mut market = Account::<'info, Market>::try_from(market)?;
            {
                let meta = &market.meta;
                let mint = Account::<Mint>::try_from(&mints[idx])?;
                require_eq!(
                    meta.market_token_mint,
                    mint.key(),
                    DataStoreError::InvalidSwapPath
                );
                require_eq!(
                    mint.key(),
                    expected_mints[idx],
                    DataStoreError::InvalidSwapPath
                );
                require!(
                    meta.long_token_mint != meta.short_token_mint,
                    DataStoreError::InvalidSwapPath
                );
                let (is_token_in_long, token_out) = if token_in == meta.long_token_mint {
                    (true, meta.short_token_mint)
                } else if token_in == meta.short_token_mint {
                    (false, meta.long_token_mint)
                } else {
                    return Err(DataStoreError::InvalidSwapPath.into());
                };
                let long_token_price = self
                    .oracle
                    .primary
                    .get(&meta.long_token_mint)
                    .ok_or(DataStoreError::InvalidArgument)?;
                let short_token_price = self
                    .oracle
                    .primary
                    .get(&meta.short_token_mint)
                    .ok_or(DataStoreError::InvalidArgument)?;
                let report = market
                    .as_market(&mint)
                    .swap(
                        is_token_in_long,
                        amount.into(),
                        long_token_price.max.to_unit_price(),
                        short_token_price.max.to_unit_price(),
                    )
                    .map_err(GmxCoreError::from)?
                    .execute()
                    .map_err(GmxCoreError::from)?;
                token_in = token_out;
                amount = (*report.token_out_amount())
                    .try_into()
                    .map_err(|_| DataStoreError::AmountOverflow)?;
                msg!("{:?}", report);
            }
            // FIXME: Is this needed?
            market.exit(&ID)?;
        }
        require_eq!(token_in, final_token, DataStoreError::InvalidSwapPath);
        Ok(amount)
    }

    fn perform_deposit(
        &mut self,
        meta: &MarketMeta,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let long_price = self
            .oracle
            .primary
            .get(&meta.long_token_mint)
            .ok_or(DataStoreError::InvalidArgument)?
            .max
            .to_unit_price();
        let short_price = self
            .oracle
            .primary
            .get(&meta.short_token_mint)
            .ok_or(DataStoreError::InvalidArgument)?
            .max
            .to_unit_price();
        self.market
            .as_market(&self.market_token_mint)
            .enable_transfer(self.token_program.to_account_info(), &self.store)
            .with_receiver(self.receiver.to_account_info())
            .deposit(
                long_amount.into(),
                short_amount.into(),
                long_price,
                short_price,
            )
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(|err| {
                msg!(&err.to_string());
                GmxCoreError::from(err)
            })?;
        Ok(())
    }
}
