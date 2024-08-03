use std::collections::BTreeSet;

use anchor_lang::prelude::*;
use gmsol_store::{
    cpi::{
        accounts::{
            ExecuteOrder, GetValidatedMarketMeta, InitializeOrder, PrepareAssociatedTokenAccount,
            RemoveOrder,
        },
        execute_order, initialize_order, prepare_associated_token_account, remove_order,
    },
    states::{
        order::{OrderKind, OrderParams, TransferOut},
        MarketMeta, NonceBytes, Position, TokenMapHeader, TokenMapLoader,
    },
    utils::{WithOracle, WithOracleExt, WithStore},
};

use crate::{
    utils::{token_records, ControllerSeeds},
    ExchangeError,
};

use super::TransferOutUtils;

pub(crate) struct PositionCutUtils<'a, 'info> {
    pub(crate) authority: AccountInfo<'info>,
    pub(crate) controller: AccountInfo<'info>,
    pub(crate) store: AccountInfo<'info>,
    pub(crate) token_map: &'a AccountLoader<'info, TokenMapHeader>,
    pub(crate) oracle: AccountInfo<'info>,
    pub(crate) market: AccountInfo<'info>,
    pub(crate) owner: AccountInfo<'info>,
    pub(crate) market_token_mint: AccountInfo<'info>,
    pub(crate) long_token_mint: AccountInfo<'info>,
    pub(crate) short_token_mint: AccountInfo<'info>,
    pub(crate) position: &'a AccountLoader<'info, Position>,
    pub(crate) order: AccountInfo<'info>,
    pub(crate) long_token_account: AccountInfo<'info>,
    pub(crate) long_token_vault: AccountInfo<'info>,
    pub(crate) short_token_account: AccountInfo<'info>,
    pub(crate) short_token_vault: AccountInfo<'info>,
    pub(crate) claimable_long_token_account_for_user: AccountInfo<'info>,
    pub(crate) claimable_short_token_account_for_user: AccountInfo<'info>,
    pub(crate) claimable_pnl_token_account_for_holding: AccountInfo<'info>,
    pub(crate) event_authority: AccountInfo<'info>,
    pub(crate) store_program: AccountInfo<'info>,
    pub(crate) price_provider: AccountInfo<'info>,
    pub(crate) token_program: AccountInfo<'info>,
    pub(crate) associated_token_program: AccountInfo<'info>,
    pub(crate) system_program: AccountInfo<'info>,
    pub(crate) remaining_accounts: &'info [AccountInfo<'info>],
}

#[derive(Clone)]
pub(crate) enum PositionCut {
    Liquidate,
    AutoDeleverage(u128),
}

impl PositionCut {
    fn size_delta_usd(&self, size_in_usd: u128) -> u128 {
        match self {
            Self::Liquidate => size_in_usd,
            Self::AutoDeleverage(delta) => size_in_usd.min(*delta),
        }
    }

    fn to_order_kind(&self) -> OrderKind {
        match self {
            Self::Liquidate => OrderKind::Liquidation,
            Self::AutoDeleverage(_) => OrderKind::AutoDeleveraging,
        }
    }
}

impl<'a, 'info> PositionCutUtils<'a, 'info> {
    /// CHECK: The caller must check all the used accounts before the execution.
    #[inline(never)]
    pub(crate) fn unchecked_execute(
        &mut self,
        kind: PositionCut,
        recent_timestamp: i64,
        nonce: NonceBytes,
        controller: &ControllerSeeds,
    ) -> Result<(u64, bool)> {
        let meta = self.get_validated_market_meta()?;
        let tokens = meta.ordered_tokens();
        let cost = self.prepare_token_accounts()?;
        self.initialize_order(&kind, &meta, &tokens, nonce, controller)?;
        let (should_remove_position, mut transfer_out) = self.execute_order(
            &meta,
            controller,
            recent_timestamp,
            tokens.into_iter().collect(),
        )?;
        self.process_transfer_out(&meta, controller, &mut transfer_out)?;
        self.remove_order(controller)?;
        Ok((cost, should_remove_position))
    }

    #[inline(never)]
    fn execute_order(
        &mut self,
        meta: &MarketMeta,
        controller: &ControllerSeeds,
        recent_timestamp: i64,
        tokens: Vec<Pubkey>,
    ) -> Result<(bool, Box<TransferOut>)> {
        let (should_remove_position, transfer_out) = self.with_oracle_prices(
            tokens,
            self.remaining_accounts,
            &controller.as_seeds(),
            |this, _| {
                Ok(execute_order(
                    this.execute_order_ctx(meta)?
                        .with_signer(&[&controller.as_seeds()]),
                    recent_timestamp,
                    true,
                )?
                .get())
            },
        )?;
        require!(transfer_out.executed, ExchangeError::InvalidArgument);
        Ok((should_remove_position, transfer_out))
    }

    #[inline(never)]
    fn initialize_order(
        &self,
        kind: &PositionCut,
        meta: &MarketMeta,
        tokens: &BTreeSet<Pubkey>,
        nonce: NonceBytes,
        controller: &ControllerSeeds,
    ) -> Result<()> {
        let output_token_account = self.output_token_accounts(meta)?.1;
        let secondary_output_token_account = self.secondary_output_token_accounts()?.1;
        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
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
        );

        let (owner, params, output_token) = self.get_params(kind)?;

        initialize_order(
            ctx.with_signer(&[&controller.as_seeds()]),
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

    fn get_params(&self, kind: &PositionCut) -> Result<(Pubkey, OrderParams, Pubkey)> {
        let position = self.position.load()?;

        let size_delta_usd = kind.size_delta_usd(position.state.size_in_usd);
        require_gt!(size_delta_usd, 0, ExchangeError::InvalidArgument);

        let params = OrderParams {
            kind: kind.to_order_kind(),
            min_output_amount: 0,
            size_delta_usd,
            initial_collateral_delta_amount: 0,
            acceptable_price: None,
            trigger_price: None,
            is_long: position.try_is_long()?,
        };

        Ok((position.owner, params, position.collateral_token))
    }

    fn is_output_token_long(&self, meta: &MarketMeta) -> Result<bool> {
        let position = self.position.load()?;
        meta.to_token_side(&position.collateral_token)
    }

    fn is_pnl_token_long(&self) -> Result<bool> {
        let position = self.position.load()?;
        position.try_is_long()
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

    fn prepare_token_accounts(&self) -> Result<u64> {
        let before = self.authority.lamports();
        prepare_associated_token_account(self.prepare_token_account_ctx(true))?;
        prepare_associated_token_account(self.prepare_token_account_ctx(false))?;
        let after = self.authority.lamports();
        let cost = before.saturating_sub(after);
        msg!("prepared token accounts, cost = {}", cost);
        Ok(cost)
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
            self.store_program.to_account_info(),
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

    fn get_validated_market_meta(&self) -> Result<MarketMeta> {
        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            GetValidatedMarketMeta {
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
            },
        );
        let meta = gmsol_store::cpi::get_validated_market_meta(ctx)?.get();
        self.validate_mints(&meta)?;
        Ok(meta)
    }

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

    fn execute_order_ctx(
        &self,
        meta: &MarketMeta,
    ) -> Result<CpiContext<'_, '_, '_, 'info, ExecuteOrder<'info>>> {
        let (output_token_vault, output_token_account) = self.output_token_accounts(meta)?;
        let (secondary_output_token_vault, secondary_output_token_account) =
            self.secondary_output_token_accounts()?;
        Ok(CpiContext::new(
            self.store_program.to_account_info(),
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
                program: self.store_program.to_account_info(),
            },
        ))
    }

    fn process_transfer_out(
        &self,
        meta: &MarketMeta,
        controller: &ControllerSeeds,
        transfer_out: &mut TransferOut,
    ) -> Result<()> {
        // CHECK: the transfer out amounts should have been validated during the execution.
        self.transfer_out_utils(meta)?
            .unchecked_process(controller, transfer_out)?;
        Ok(())
    }

    fn transfer_out_utils(&self, meta: &MarketMeta) -> Result<TransferOutUtils<'info>> {
        let (output_token_vault, output_token_account) = self.output_token_accounts(meta)?;
        let (secondary_output_token_vault, secondary_output_token_account) =
            self.secondary_output_token_accounts()?;
        Ok(TransferOutUtils {
            store_program: self.store_program.to_account_info(),
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

    #[inline(never)]
    fn remove_order(&self, controller: &ControllerSeeds) -> Result<()> {
        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            RemoveOrder {
                payer: self.authority.to_account_info(),
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                order: self.order.to_account_info(),
                user: self.owner.to_account_info(),
                system_program: self.system_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.store_program.to_account_info(),
            },
        );
        remove_order(
            ctx.with_signer(&[&controller.as_seeds()]),
            0,
            "position cut order executed".to_string(),
        )
    }
}

impl<'a, 'info> WithStore<'info> for PositionCutUtils<'a, 'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.clone()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.clone()
    }
}

impl<'a, 'info> WithOracle<'info> for PositionCutUtils<'a, 'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.clone()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.clone()
    }

    fn token_map(&self) -> AccountInfo<'info> {
        self.token_map.to_account_info()
    }

    fn controller(&self) -> AccountInfo<'info> {
        self.controller.clone()
    }
}
