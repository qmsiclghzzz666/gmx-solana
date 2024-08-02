use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::{
        accounts::{GetValidatedMarketMeta, RemovePosition},
        get_validated_market_meta, remove_position,
    },
    program::GmsolStore,
    states::{NonceBytes, Oracle, Position, PriceProvider, Store, TokenMapHeader},
    utils::{Authentication, WithOracle, WithOracleExt, WithStore},
};

use crate::{
    order::utils::PositionCut,
    states::{ActionDisabledFlag, Controller, DomainDisabledFlag},
    utils::ControllerSeeds,
    ExchangeError,
};

use super::utils::PositionCutUtils;

/// The accounts definitions for [`auto_deleverage`](crate::gmsol_exchange::auto_deleverage).
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::auto_deleverage).*
#[derive(Accounts)]
pub struct AutoDeleverage<'info> {
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
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    pub market_token_mint: Account<'info, Mint>,
    pub long_token_mint: Account<'info, Mint>,
    pub short_token_mint: Account<'info, Mint>,
    /// CHECK: only used to invoke CPI and then checked and initilized by it.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// Position to be auto-deleveraged.
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

/// Auto-deleverage the given position.
///
/// # CHECK
/// - This instruction can only be called by the `ORDER_KEEPER` (maybe `ADL_KEEPER` in the future).
pub(crate) fn unchecked_auto_deleverage<'info>(
    ctx: Context<'_, '_, 'info, 'info, AutoDeleverage<'info>>,
    size_delta_usd: u128,
    recent_timestamp: i64,
    nonce: NonceBytes,
    execution_fee: u64,
) -> Result<()> {
    ctx.accounts.controller.load()?.validate_feature_enabled(
        DomainDisabledFlag::AutoDeleveraging,
        ActionDisabledFlag::ExecuteOrder,
    )?;

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    // CHECK: Refer to the documentation of the `liquidate` instruction for details on the account checks.
    let (cost, should_remove_position) = ctx
        .accounts
        .position_cut_utils(ctx.remaining_accounts)
        .unchecked_execute(
            PositionCut::AutoDeleverage(size_delta_usd),
            recent_timestamp,
            nonce,
            &controller,
        )?;

    // Remove position.
    if should_remove_position {
        ctx.accounts
            .remove_position(&controller, cost, execution_fee)?;
    }
    Ok(())
}

impl<'info> WithStore<'info> for AutoDeleverage<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for AutoDeleverage<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> AutoDeleverage<'info> {
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
            market: self.market.to_account_info(),
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

/// The accounts definition for [`update_adl_state`](crate::gmsol_exchange::update_adl_state).
///
/// *[See also the documentation for the instruction.](crate::gmsol_exchange::update_adl_state)*
#[derive(Accounts)]
pub struct UpdateAdlState<'info> {
    /// The address authorized to execute this instruction.
    pub authority: Signer<'info>,
    /// The controller.
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
    /// The store.
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// The authoritzed token map.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// Buffer for the oracle prices.
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// The market to update the ADL state.
    /// CHECK: check by the CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The price provider program.
    pub price_provider: Interface<'info, PriceProvider>,
}

/// # CHECK
/// - This instruction can only be called by the `ORDER_KEEPER` (maybe `ADL_KEEPER` in the future).
pub(crate) fn unchecked_update_adl_state<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
    is_long: bool,
) -> Result<()> {
    // Get tokens.
    let tokens = get_validated_market_meta(CpiContext::new(
        ctx.accounts.store_program.to_account_info(),
        GetValidatedMarketMeta {
            store: ctx.accounts.store.to_account_info(),
            market: ctx.accounts.market.to_account_info(),
        },
    ))?
    .get()
    .ordered_tokens();

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    ctx.accounts.with_oracle_prices(
        tokens.into_iter().collect(),
        ctx.remaining_accounts,
        &controller.as_seeds(),
        |accounts, _| accounts.update_adl_state(&controller, is_long),
    )
}

impl<'info> UpdateAdlState<'info> {
    fn update_adl_state(&self, controller: &ControllerSeeds, is_long: bool) -> Result<()> {
        use gmsol_store::cpi;

        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            cpi::accounts::UpdateAdlState {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                oracle: self.oracle.to_account_info(),
                market: self.market.to_account_info(),
            },
        );

        cpi::update_adl_state(ctx.with_signer(&[&controller.as_seeds()]), is_long)
    }
}

impl<'info> WithStore<'info> for UpdateAdlState<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> Authentication<'info> for UpdateAdlState<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> WithOracle<'info> for UpdateAdlState<'info> {
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
