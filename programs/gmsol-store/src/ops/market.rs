use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use gmsol_model::{LiquidityMarketMutExt, PositionImpactMarketMutExt};
use typed_builder::TypedBuilder;

use crate::{
    states::{
        common::swap::SwapParams,
        market::{
            revertible::{
                liquidity_market::RevertibleLiquidityMarket2,
                swap_market::{SwapDirection, SwapMarkets},
                Revertible,
            },
            utils::ValidateMarketBalances,
            HasMarketMeta,
        },
        Market, Oracle, Store,
    },
    CoreError, ModelError,
};

/// Operation for transferring funds into market valut.
#[derive(TypedBuilder)]
pub(crate) struct MarketTransferInOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    from: AccountInfo<'info>,
    from_authority: AccountInfo<'info>,
    vault: &'a Account<'info, TokenAccount>,
    amount: u64,
    token_program: AccountInfo<'info>,
    signer_seeds: &'a [&'a [u8]],
}

impl<'a, 'info> MarketTransferInOperation<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use anchor_spl::token;

        self.market.load()?.validate(&self.store.key())?;

        let amount = self.amount;
        if amount != 0 {
            token::transfer(
                CpiContext::new(
                    self.token_program,
                    token::Transfer {
                        from: self.from,
                        to: self.vault.to_account_info(),
                        authority: self.from_authority,
                    },
                )
                .with_signer(&[self.signer_seeds]),
                amount,
            )?;
            let token = &self.vault.mint;
            self.market
                .load_mut()?
                .record_transferred_in_by_token(token, amount)?;
        }

        Ok(())
    }
}

/// Operation for transferring funds out of market vault.
#[derive(TypedBuilder)]
pub(crate) struct MarketTransferOutOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    amount: u64,
    decimals: u8,
    to: AccountInfo<'info>,
    token_mint: AccountInfo<'info>,
    vault: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'a, 'info> MarketTransferOutOperation<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use crate::utils::internal::TransferUtils;

        {
            let market = self.market.load()?;
            let meta = market.validated_meta(&self.store.key())?;
            require!(
                meta.is_collateral_token(&self.token_mint.key()),
                CoreError::InvalidCollateralToken
            );
        }

        let amount = self.amount;
        if amount != 0 {
            let decimals = self.decimals;
            TransferUtils::new(
                self.token_program.to_account_info(),
                self.store,
                self.token_mint.to_account_info(),
            )
            .transfer_out(self.vault.to_account_info(), self.to, amount, decimals)?;
            let token = &self.token_mint.key();
            self.market
                .load_mut()?
                .record_transferred_out_by_token(token, amount)?;
        }

        Ok(())
    }
}

/// Revertible Liquidity Market Operation.
pub struct RevertibleLiquidityMarketOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    oracle: &'a Oracle,
    market: &'a AccountLoader<'info, Market>,
    market_token_mint: &'a mut Account<'info, Mint>,
    token_program: AccountInfo<'info>,
    swap: &'a SwapParams,
    swap_markets: Vec<AccountLoader<'info, Market>>,
}

impl<'a, 'info> RevertibleLiquidityMarketOperation<'a, 'info> {
    pub(crate) fn new(
        store: &'a AccountLoader<'info, Store>,
        oracle: &'a Oracle,
        market: &'a AccountLoader<'info, Market>,
        market_token_mint: &'a mut Account<'info, Mint>,
        token_program: AccountInfo<'info>,
        swap: &'a SwapParams,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<Self> {
        let swap_markets =
            swap.unpack_markets_for_swap(&market_token_mint.key(), remaining_accounts)?;

        Ok(Self {
            store,
            oracle,
            market,
            market_token_mint,
            token_program,
            swap,
            swap_markets,
        })
    }
}

impl<'a, 'info> RevertibleLiquidityMarketOperation<'a, 'info> {
    /// Deposit into the market.
    ///
    /// # CHECK
    ///
    /// # Errors
    #[inline(never)]
    pub(crate) fn unchecked_deposit<'c>(
        &'c mut self,
        market_token_receiver: &'c AccountInfo<'info>,
        initial_tokens: (Option<Pubkey>, Option<Pubkey>),
        initial_amounts: (u64, u64),
        min_market_token_amount: u64,
    ) -> Result<ExecutedDeposit<'c, 'info>> {
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket2::from_revertible_market(
            self.market.try_into()?,
            self.market_token_mint,
            &self.token_program,
            self.store,
        )?
        .enable_mint(market_token_receiver);
        let mut swap_markets = SwapMarkets::new(
            &self.store.key(),
            &self.swap_markets,
            Some(&current_market_token),
        )?;

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
                self.swap,
                expected_token_outs,
                initial_tokens,
                initial_amounts,
            )?
        };

        // Perform the deposit.
        let minted = {
            let prices = self.oracle.market_prices(&market)?;
            let report = market
                .deposit(long_token_amount.into(), short_token_amount.into(), prices)
                .and_then(|d| d.execute())
                .map_err(ModelError::from)?;
            market.validate_market_balances(0, 0)?;

            let minted: u64 = (*report.minted())
                .try_into()
                .map_err(|_| error!(CoreError::TokenAmountOverflow))?;

            require_gte!(
                minted,
                min_market_token_amount,
                CoreError::InsufficientOutputAmount
            );

            msg!("[Deposit] executed: {:?}", report);

            minted
        };

        Ok(ExecutedDeposit {
            minted_amount: minted,
            market,
            swap_markets,
        })
    }
}

#[must_use = "Revertible operation must be committed to take effect"]
pub(crate) struct ExecutedDeposit<'a, 'info> {
    pub(crate) minted_amount: u64,
    pub(crate) market: RevertibleLiquidityMarket2<'a, 'info>,
    pub(crate) swap_markets: SwapMarkets<'a>,
}

impl<'a, 'info> ExecutedDeposit<'a, 'info> {
    #[allow(dead_code)]
    pub(crate) fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl<'a, 'info> Revertible for ExecutedDeposit<'a, 'info> {
    fn commit(self) {
        self.market.commit();
        self.swap_markets.commit();
    }
}
