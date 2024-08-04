use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::{accounts::RemovePosition, remove_position},
    program::GmsolStore,
    states::{Market, NonceBytes, Oracle, Position, PriceProvider, Store, TokenMapHeader},
    utils::{Authentication, WithStore},
};

use crate::{
    order::utils::PositionCut,
    states::{ActionDisabledFlag, Controller, DomainDisabledFlag},
    utils::ControllerSeeds,
    ExchangeError,
};

use super::utils::PositionCutUtils;

/// The accounts definitions for [`liquidate`](crate::gmsol_exchange::liquidate).
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::liquidate).*
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
    /// Controller.
    #[account(
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Buffer for oracle prices.
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
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
    ctx.accounts.controller.load()?.validate_feature_enabled(
        DomainDisabledFlag::Liquidation,
        ActionDisabledFlag::ExecuteOrder,
    )?;

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    // CHECK: Refer to the documentation of the `liquidate` instruction for details on the account checks.
    let (cost, should_remove_position) = ctx
        .accounts
        .position_cut_utils(ctx.remaining_accounts)
        .unchecked_execute(PositionCut::Liquidate, recent_timestamp, nonce, &controller)?;

    // Remove position.
    require!(should_remove_position, ExchangeError::InvalidArgument);
    ctx.accounts
        .remove_position(&controller, cost, execution_fee)?;
    Ok(())
}

impl<'info> WithStore<'info> for Liquidate<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for Liquidate<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> Liquidate<'info> {
    fn position_cut_utils(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> PositionCutUtils<'_, 'info> {
        PositionCutUtils {
            authority: self.authority.to_account_info(),
            controller: self.controller.to_account_info(),
            store: self.store.to_account_info(),
            token_map: &self.token_map,
            oracle: self.oracle.to_account_info(),
            market: &self.market,
            owner: self.owner.to_account_info(),
            market_token_mint: self.market_token_mint.to_account_info(),
            long_token_mint: self.long_token_mint.to_account_info(),
            short_token_mint: self.short_token_mint.to_account_info(),
            position: &self.position,
            order: self.order.to_account_info(),
            long_token_account: self.long_token_account.to_account_info(),
            long_token_vault: self.long_token_vault.to_account_info(),
            short_token_account: self.short_token_account.to_account_info(),
            short_token_vault: self.short_token_vault.to_account_info(),
            claimable_long_token_account_for_user: self
                .claimable_long_token_account_for_user
                .to_account_info(),
            claimable_short_token_account_for_user: self
                .claimable_short_token_account_for_user
                .to_account_info(),
            claimable_pnl_token_account_for_holding: self
                .claimable_pnl_token_account_for_holding
                .to_account_info(),
            event_authority: self.event_authority.to_account_info(),
            store_program: self.data_store_program.to_account_info(),
            price_provider: self.price_provider.to_account_info(),
            token_program: self.token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            remaining_accounts,
        }
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
