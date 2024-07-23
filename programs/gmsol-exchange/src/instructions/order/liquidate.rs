use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::{
        accounts::{ExecuteOrder, GetValidatedMarketMeta, InitializeOrder},
        execute_order, initialize_order,
    },
    program::GmsolStore,
    states::{
        order::{OrderKind, OrderParams, TransferOut},
        MarketMeta, NonceBytes, Oracle, Position, PriceProvider, Store, TokenMapHeader,
        TokenMapLoader,
    },
    utils::{Authentication, WithOracle, WithOracleExt},
    StoreError,
};

use crate::{token_records, utils::ControllerSeeds, ExchangeError};

use super::utils::TransferOutUtils;

/// The accounts definitions for [`liquidate`](gmsol_exchange::gmsol_exchange::liquidate).
#[derive(Accounts)]
pub struct Liquidate<'info> {
    /// The authority of this instruction.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Buffer for oracle prices.
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: it will be checked by the related CPI.
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    /// CHECK: only used to invoke CPI and then checked and initilized by it.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// Position to be liquidated.
    #[account(mut)]
    pub position: AccountLoader<'info, Position>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub final_output_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub secondary_output_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub final_output_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub secondary_output_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub long_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub short_token_vault: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub long_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub short_token_account: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_long_token_account_for_user: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_short_token_account_for_user: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub claimable_pnl_token_account_for_holding: UncheckedAccount<'info>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
    pub token_program: Program<'info, Token>,
    pub price_provider: Interface<'info, PriceProvider>,
    pub system_program: Program<'info, System>,
}

/// Liquidate the given position.
///
/// # CHECK
/// - This instruction can only be called by the `ORDER_KEEPER` (maybe `LIQUIDATE_KEEPER` in the future).
pub(crate) fn unchecked_liquidate<'info>(
    ctx: Context<'_, '_, 'info, 'info, Liquidate<'info>>,
    recent_timestamp: i64,
    nonce: NonceBytes,
) -> Result<()> {
    let store_key = &ctx.accounts.store.key();
    let controller = ControllerSeeds::find(store_key);
    let meta = ctx.accounts.get_validated_market_meta()?;
    let tokens = vec![
        meta.index_token_mint,
        meta.long_token_mint,
        meta.short_token_mint,
    ];
    ctx.accounts.initialize_order(&controller, nonce, &tokens)?;
    let remaining_accounts = ctx.remaining_accounts;
    let transfer_out =
        ctx.accounts
            .execute_order(&controller, recent_timestamp, tokens, remaining_accounts)?;
    Ok(())
}

impl<'info> Authentication<'info> for Liquidate<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }

    fn data_store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> WithOracle<'info> for Liquidate<'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_map(&self) -> AccountInfo<'info> {
        self.token_map.to_account_info()
    }

    fn controller(&self) -> AccountInfo<'info> {
        self.controller.to_account_info()
    }
}

impl<'info> Liquidate<'info> {
    fn initialize_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitializeOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeOrder {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                payer: self.authority.to_account_info(),
                order: self.order.to_account_info(),
                position: Some(self.position.to_account_info()),
                market: self.market.to_account_info(),
                initial_collateral_token_account: None,
                initial_collateral_token_vault: None,
                final_output_token_account: Some(self.final_output_token_account.to_account_info()),
                secondary_output_token_account: Some(
                    self.secondary_output_token_account.to_account_info(),
                ),
                long_token_account: self.long_token_account.to_account_info(),
                short_token_account: self.short_token_account.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }

    fn initialize_order(
        &self,
        controller: &ControllerSeeds,
        nonce: NonceBytes,
        tokens: &[Pubkey],
    ) -> Result<()> {
        let position = self.position.load()?;
        let size_in_usd = position.state.size_in_usd;
        require_gt!(size_in_usd, 0, StoreError::InvalidPosition);
        let params = OrderParams {
            kind: OrderKind::Liquidation,
            min_output_amount: 0,
            size_delta_usd: size_in_usd,
            initial_collateral_delta_amount: 0,
            acceptable_price: None,
            is_long: position.is_long()?,
        };
        initialize_order(
            self.initialize_order_ctx()
                .with_signer(&[&controller.as_seeds()]),
            position.owner,
            nonce,
            token_records(&self.token_map.load_token_map()?, tokens.iter().copied())?,
            Default::default(),
            params,
            position.collateral_token,
            self.authority.key(),
        )?;
        Ok(())
    }

    fn get_validated_market_meta(&self) -> Result<MarketMeta> {
        let ctx = CpiContext::new(
            self.data_store_program.to_account_info(),
            GetValidatedMarketMeta {
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
            },
        );
        let meta = gmsol_store::cpi::get_validated_market_meta(ctx)?.get();
        Ok(meta)
    }

    fn execute_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, ExecuteOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            ExecuteOrder {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                oracle: self.oracle.to_account_info(),
                order: self.order.to_account_info(),
                market: self.market.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                position: Some(self.position.to_account_info()),
                final_output_token_vault: Some(self.final_output_token_vault.to_account_info()),
                secondary_output_token_vault: Some(
                    self.secondary_output_token_vault.to_account_info(),
                ),
                final_output_token_account: Some(self.final_output_token_account.to_account_info()),
                secondary_output_token_account: Some(
                    self.secondary_output_token_account.to_account_info(),
                ),
                long_token_vault: self.long_token_vault.to_account_info(),
                short_token_vault: self.short_token_vault.to_account_info(),
                long_token_account: self.long_token_account.to_account_info(),
                short_token_account: self.short_token_account.to_account_info(),
                claimable_long_token_account_for_user: Some(
                    self.claimable_long_token_account_for_user.to_account_info(),
                ),
                claimable_short_token_account_for_user: Some(
                    self.claimable_short_token_account_for_user
                        .to_account_info(),
                ),
                claimable_pnl_token_account_for_holding: Some(
                    self.claimable_pnl_token_account_for_holding
                        .to_account_info(),
                ),
                token_program: self.token_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }

    fn execute_order(
        &mut self,
        controller: &ControllerSeeds,
        recent_timestamp: i64,
        tokens: Vec<Pubkey>,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<Box<TransferOut>> {
        let (should_remove_position, transfer_out) = self.with_oracle_prices(
            tokens,
            remaining_accounts,
            &controller.as_seeds(),
            |accounts, _| {
                Ok(execute_order(accounts.execute_order_ctx(), recent_timestamp, true)?.get())
            },
        )?;
        require!(should_remove_position, StoreError::InvalidArgument);
        Ok(transfer_out)
    }
}
