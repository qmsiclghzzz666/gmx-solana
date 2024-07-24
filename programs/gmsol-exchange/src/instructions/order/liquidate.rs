use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::{
        accounts::{
            ExecuteOrder, GetValidatedMarketMeta, InitializeOrder, PrepareAssociatedTokenAccount,
            RemoveOrder, RemovePosition,
        },
        execute_order, initialize_order, prepare_associated_token_account, remove_order,
        remove_position,
    },
    program::GmsolStore,
    states::{
        order::{OrderKind, OrderParams, TransferOut},
        MarketMeta, NonceBytes, Oracle, Position, PriceProvider, Store, TokenMapHeader,
        TokenMapLoader,
    },
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::{utils::token_records, utils::ControllerSeeds, ExchangeError};

use super::utils::TransferOutUtils;

/// The accounts definitions for [`liquidate`](gmsol_exchange::gmsol_exchange::liquidate).
#[derive(Accounts)]
pub struct Liquidate<'info> {
    /// The authority of this instruction.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The owner of the position.
    /// CHECK: only used to reference to the owner
    /// and receive fund.
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
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
    pub market_token_mint: Account<'info, Mint>,
    pub long_token_mint: Account<'info, Mint>,
    pub short_token_mint: Account<'info, Mint>,
    /// CHECK: only used to invoke CPI and then checked and initilized by it.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// Position to be liquidated.
    #[account(
        mut,
        constraint = position.load()?.store == store.key() @ ExchangeError::InvalidArgument,
        constraint = position.load()?.owner == *owner.key @ ExchangeError::InvalidArgument,
    )]
    pub position: AccountLoader<'info, Position>,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    execution_fee: u64,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);
    let meta = ctx.accounts.get_validated_market_meta()?;

    // Make sure the token mint matches.
    ctx.accounts.validate_mints(&meta)?;

    let tokens = [
        meta.index_token_mint,
        meta.long_token_mint,
        meta.short_token_mint,
    ]
    .into();
    // Prepare token accounts.
    let cost = {
        let payer = &ctx.accounts.authority;
        let before_lamports = payer.lamports();
        ctx.accounts.prepare_token_accounts()?;
        let after_lamports = payer.lamports();
        before_lamports.saturating_sub(after_lamports)
    };
    msg!("prepared token accounts, cost = {}", cost);

    // Initialize order.
    ctx.accounts
        .initialize_order(&meta, &controller, nonce, &tokens)?;

    // Execute order.
    {
        let remaining_accounts = ctx.remaining_accounts;
        let transfer_out = ctx.accounts.execute_order(
            &meta,
            &controller,
            recent_timestamp,
            tokens.into_iter().collect(),
            remaining_accounts,
        )?;
        ctx.accounts
            .process_transfer_out(&meta, &controller, &transfer_out)?;
    }

    // Remove order.
    ctx.accounts.remove_order(&controller)?;

    // Remove position.
    ctx.accounts
        .remove_position(&controller, cost, execution_fee)?;
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
    fn validate_mints(&self, meta: &MarketMeta) -> Result<()> {
        require_eq!(
            meta.long_token_mint,
            self.long_token_mint.key(),
            ExchangeError::InvalidArgument
        );
        require_eq!(
            meta.short_token_mint,
            self.short_token_mint.key(),
            ExchangeError::InvalidArgument
        );
        Ok(())
    }

    fn initialize_order_ctx(
        &self,
        meta: &MarketMeta,
    ) -> Result<CpiContext<'_, '_, '_, 'info, InitializeOrder<'info>>> {
        let output_token_account = self.output_token_accounts(meta)?.1;
        let secondary_output_token_account = self.secondary_output_token_accounts()?.1;
        Ok(CpiContext::new(
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
                final_output_token_account: Some(output_token_account),
                secondary_output_token_account: Some(secondary_output_token_account),
                long_token_account: self.long_token_account.to_account_info(),
                short_token_account: self.short_token_account.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        ))
    }

    fn initialize_order(
        &self,
        meta: &MarketMeta,
        controller: &ControllerSeeds,
        nonce: NonceBytes,
        tokens: &BTreeSet<Pubkey>,
    ) -> Result<()> {
        let position = self.position.load()?;
        let size_in_usd = position.state.size_in_usd;
        let is_long = position.is_long()?;
        let owner = position.owner;
        let output_token = position.collateral_token;
        drop(position);

        require_gt!(size_in_usd, 0, ExchangeError::InvalidArgument);
        let params = OrderParams {
            kind: OrderKind::Liquidation,
            min_output_amount: 0,
            size_delta_usd: size_in_usd,
            initial_collateral_delta_amount: 0,
            acceptable_price: None,
            is_long,
        };
        initialize_order(
            self.initialize_order_ctx(meta)?
                .with_signer(&[&controller.as_seeds()]),
            owner,
            nonce,
            token_records(&self.token_map.load_token_map()?, tokens)?,
            Default::default(),
            params,
            output_token,
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

    fn is_output_token_long(&self, meta: &MarketMeta) -> Result<bool> {
        let position = self.position.load()?;
        meta.to_token_side(&position.collateral_token)
    }

    fn is_pnl_token_long(&self) -> Result<bool> {
        let position = self.position.load()?;
        position.is_long()
    }

    fn output_token_accounts(
        &self,
        meta: &MarketMeta,
    ) -> Result<(AccountInfo<'info>, AccountInfo<'info>)> {
        if self.is_output_token_long(meta)? {
            Ok((
                self.long_token_vault.to_account_info(),
                self.long_token_account.to_account_info(),
            ))
        } else {
            Ok((
                self.short_token_vault.to_account_info(),
                self.short_token_account.to_account_info(),
            ))
        }
    }

    fn secondary_output_token_accounts(&self) -> Result<(AccountInfo<'info>, AccountInfo<'info>)> {
        if self.is_pnl_token_long()? {
            Ok((
                self.long_token_vault.to_account_info(),
                self.long_token_account.to_account_info(),
            ))
        } else {
            Ok((
                self.short_token_vault.to_account_info(),
                self.short_token_account.to_account_info(),
            ))
        }
    }

    fn execute_order_ctx(
        &self,
        meta: &MarketMeta,
    ) -> Result<CpiContext<'_, '_, '_, 'info, ExecuteOrder<'info>>> {
        let (output_token_vault, output_token_account) = self.output_token_accounts(meta)?;
        let (secondary_output_token_vault, secondary_output_token_account) =
            self.secondary_output_token_accounts()?;
        Ok(CpiContext::new(
            self.data_store_program.to_account_info(),
            ExecuteOrder {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                oracle: self.oracle.to_account_info(),
                order: self.order.to_account_info(),
                market: self.market.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                position: Some(self.position.to_account_info()),
                final_output_token_vault: Some(output_token_vault),
                secondary_output_token_vault: Some(secondary_output_token_vault),
                final_output_token_account: Some(output_token_account),
                secondary_output_token_account: Some(secondary_output_token_account),
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
        ))
    }

    fn execute_order(
        &mut self,
        meta: &MarketMeta,
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
                Ok(execute_order(
                    accounts
                        .execute_order_ctx(meta)?
                        .with_signer(&[&controller.as_seeds()]),
                    recent_timestamp,
                    true,
                )?
                .get())
            },
        )?;
        require!(should_remove_position, ExchangeError::InvalidArgument);
        require!(transfer_out.executed, ExchangeError::InvalidArgument);
        Ok(transfer_out)
    }

    fn prepare_token_account_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, PrepareAssociatedTokenAccount<'info>> {
        let (mint, account) = if is_long_token {
            (
                self.long_token_mint.to_account_info(),
                self.long_token_account.to_account_info(),
            )
        } else {
            (
                self.short_token_mint.to_account_info(),
                self.short_token_account.to_account_info(),
            )
        };
        CpiContext::new(
            self.data_store_program.to_account_info(),
            PrepareAssociatedTokenAccount {
                payer: self.authority.to_account_info(),
                owner: self.owner.to_account_info(),
                mint,
                account,
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
            },
        )
    }

    fn prepare_token_accounts(&self) -> Result<()> {
        prepare_associated_token_account(self.prepare_token_account_ctx(true))?;
        prepare_associated_token_account(self.prepare_token_account_ctx(false))?;
        Ok(())
    }

    fn transfer_out_utils(&self, meta: &MarketMeta) -> Result<TransferOutUtils<'info>> {
        let (output_token_vault, output_token_account) = self.output_token_accounts(meta)?;
        let (secondary_output_token_vault, secondary_output_token_account) =
            self.secondary_output_token_accounts()?;
        Ok(TransferOutUtils {
            store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            controller: self.controller.to_account_info(),
            market: self.market.to_account_info(),
            store: self.store.to_account_info(),
            long_token_vault: self.long_token_vault.to_account_info(),
            long_token_account: self.long_token_account.to_account_info(),
            short_token_vault: self.short_token_vault.to_account_info(),
            short_token_account: self.short_token_account.to_account_info(),
            final_output_token_account: Some(output_token_account),
            final_output_token_vault: Some(output_token_vault),
            final_output_market: self.market.to_account_info(),
            secondary_output_token_account: Some(secondary_output_token_account),
            secondary_output_token_vault: Some(secondary_output_token_vault),
            final_secondary_output_market: self.market.to_account_info(),
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
        })
    }

    fn process_transfer_out(
        &self,
        meta: &MarketMeta,
        controller: &ControllerSeeds,
        transfer_out: &TransferOut,
    ) -> Result<()> {
        // CHECK: the transfer out amounts should have been validated during the execution.
        self.transfer_out_utils(meta)?
            .unchecked_process(controller, transfer_out)?;
        Ok(())
    }

    fn remove_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveOrder {
                payer: self.authority.to_account_info(),
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                order: self.order.to_account_info(),
                user: self.owner.to_account_info(),
                system_program: self.system_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.data_store_program.to_account_info(),
            },
        )
    }

    fn remove_order(&self, controller: &ControllerSeeds) -> Result<()> {
        remove_order(
            self.remove_order_ctx()
                .with_signer(&[&controller.as_seeds()]),
            0,
            "liquidation executed".to_string(),
        )
    }

    fn remove_position_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemovePosition<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemovePosition {
                payer: self.authority.to_account_info(),
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                position: self.position.to_account_info(),
                user: self.owner.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn remove_position(
        &self,
        controller: &ControllerSeeds,
        cost: u64,
        execution_fee: u64,
    ) -> Result<()> {
        let mut refund = self.position.get_lamports();
        refund = refund.saturating_sub(cost);
        refund = refund.saturating_sub(execution_fee.min(crate::MAX_ORDER_EXECUTION_FEE));
        remove_position(
            self.remove_position_ctx()
                .with_signer(&[&controller.as_seeds()]),
            refund,
        )?;
        Ok(())
    }
}
