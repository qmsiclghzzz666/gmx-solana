use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    constants,
    events::RemoveOrderEvent,
    states::{
        common::{SwapParams, TokenRecord},
        order::{Order, OrderKind, OrderParams, Receivers, Senders, Tokens},
        position::Position,
        Market, NonceBytes, Seed, Store,
    },
    utils::internal,
    StoreError,
};

#[derive(Accounts)]
#[instruction(
    owner: Pubkey,
    nonce: [u8; 32],
    tokens_with_feed: Vec<TokenRecord>,
    swap: SwapParams,
    params: OrderParams,
    output_token: Pubkey,
)]
pub struct InitializeOrder<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        space = 8 + Order::init_space(&tokens_with_feed, &swap),
        payer = payer,
        seeds = [Order::SEED, store.key().as_ref(), owner.as_ref(), &nonce],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + Position::INIT_SPACE,
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            owner.as_ref(),
            market.load()?.meta().market_token_mint.as_ref(),
            output_token.as_ref(),
            &[params.to_position_kind()? as u8],
        ],
        bump,
        // FIXME: It cannot be check like this when the position is not initialized.
        // constraint = position.load()?.store == store.key(),
    )]
    pub position: Option<AccountLoader<'info, Position>>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut, token::authority = owner)]
    pub initial_collateral_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = initial_collateral_token_account.as_ref().expect("sender must be provided").mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_collateral_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub initial_collateral_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    #[account(token::authority = owner)]
    pub final_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(token::authority = owner)]
    pub secondary_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(token::authority = owner, token::mint = market.load()?.meta().long_token_mint)]
    pub long_token_account: Box<Account<'info, TokenAccount>>,
    #[account(token::authority = owner, token::mint = market.load()?.meta().short_token_mint)]
    pub short_token_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Initialize a new [`Order`] account.
#[allow(clippy::too_many_arguments)]
pub fn initialize_order(
    ctx: Context<InitializeOrder>,
    owner: Pubkey,
    nonce: NonceBytes,
    tokens_with_feed: Vec<TokenRecord>,
    swap: SwapParams,
    params: OrderParams,
    output_token: Pubkey,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    params.validate()?;
    let meta = *ctx.accounts.market.load()?.meta();
    // Validate and create `Tokens`.
    let tokens = match &params.kind {
        OrderKind::MarketSwap | OrderKind::LimitSwap => {
            // Validate that the `output_token` is one of the collateral tokens.
            ctx.accounts
                .market
                .load()?
                .meta()
                .validate_collateral_token(&output_token)?;
            Tokens {
                market_token: meta.market_token_mint,
                initial_collateral_token: ctx.accounts.initial_collateral_token_account()?.mint,
                output_token,
                // `secondary_output_token` is unused for swap, so just set it to `output_token` here.
                secondary_output_token: output_token,
                final_output_token: None,
            }
        }
        OrderKind::MarketIncrease | OrderKind::LimitIncrease => {
            // The validation of `output_token` is also performed by the method below.
            ctx.accounts.initialize_position_if_needed(
                &owner,
                &params,
                ctx.bumps.position,
                &output_token,
            )?;
            Tokens {
                market_token: meta.market_token_mint,
                initial_collateral_token: ctx.accounts.initial_collateral_token_account()?.mint,
                output_token,
                secondary_output_token: meta.pnl_token(params.is_long),
                final_output_token: None,
            }
        }
        OrderKind::MarketDecrease
        | OrderKind::Liquidation
        | OrderKind::AutoDeleveraging
        | OrderKind::LimitDecrease
        | OrderKind::StopLossDecrease => {
            // The validation of `output_token` is also performed by the method below.
            ctx.accounts
                .validate_position(ctx.bumps.position, &output_token)?;
            Tokens {
                market_token: meta.market_token_mint,
                initial_collateral_token: output_token,
                output_token,
                secondary_output_token: meta.pnl_token(params.is_long),
                final_output_token: Some(ctx.accounts.final_output_token_account()?.mint),
            }
        }
    };
    let (senders, receivers) = match &params.kind {
        OrderKind::MarketSwap | OrderKind::LimitSwap => (
            Senders {
                initial_collateral_token_account: Some(
                    ctx.accounts.initial_collateral_token_account()?.key(),
                ),
            },
            // The output of a swap is directly trasferred to `long_token_account` or `short_token_account`.
            Receivers {
                ui_fee: ui_fee_receiver,
                final_output_token_account: None,
                secondary_output_token_account: None,
                long_token_account: ctx.accounts.long_token_account.key(),
                short_token_account: ctx.accounts.short_token_account.key(),
            },
        ),
        OrderKind::MarketIncrease | OrderKind::LimitIncrease => (
            Senders {
                initial_collateral_token_account: Some(
                    ctx.accounts.initial_collateral_token_account()?.key(),
                ),
            },
            Receivers {
                ui_fee: ui_fee_receiver,
                final_output_token_account: None,
                secondary_output_token_account: None,
                long_token_account: ctx.accounts.long_token_account.key(),
                short_token_account: ctx.accounts.short_token_account.key(),
            },
        ),
        OrderKind::MarketDecrease
        | OrderKind::Liquidation
        | OrderKind::AutoDeleveraging
        | OrderKind::LimitDecrease
        | OrderKind::StopLossDecrease => (
            Senders {
                initial_collateral_token_account: None,
            },
            Receivers {
                ui_fee: ui_fee_receiver,
                final_output_token_account: Some(ctx.accounts.final_output_token_account()?.key()),
                secondary_output_token_account: Some(
                    ctx.accounts.secondary_output_token_account()?.key(),
                ),
                long_token_account: ctx.accounts.long_token_account.key(),
                short_token_account: ctx.accounts.short_token_account.key(),
            },
        ),
    };

    let id = ctx
        .accounts
        .market
        .load_mut()?
        .state_mut()
        .next_order_id()?;

    ctx.accounts.order.init(
        ctx.bumps.order,
        id,
        ctx.accounts.store.key(),
        &nonce,
        &ctx.accounts.market.key(),
        &owner,
        ctx.accounts.position.as_ref().map(|a| a.key()).as_ref(),
        &params,
        &tokens,
        &senders,
        &receivers,
        tokens_with_feed,
        swap,
    )?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeOrder<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> InitializeOrder<'info> {
    fn initial_collateral_token_account(&self) -> Result<&Account<'info, TokenAccount>> {
        let account = self
            .initial_collateral_token_account
            .as_ref()
            .ok_or(StoreError::MissingSender)?;
        Ok(account)
    }

    fn final_output_token_account(&self) -> Result<&Account<'info, TokenAccount>> {
        let account = self
            .final_output_token_account
            .as_ref()
            .ok_or(StoreError::MissingReceivers)?;
        Ok(account)
    }

    fn secondary_output_token_account(&self) -> Result<&Account<'info, TokenAccount>> {
        let account = self
            .secondary_output_token_account
            .as_ref()
            .ok_or(StoreError::MissingReceivers)?;
        Ok(account)
    }

    fn initialize_position_if_needed(
        &self,
        owner: &Pubkey,
        params: &OrderParams,
        bump: Option<u8>,
        output_token: &Pubkey,
    ) -> Result<()> {
        self.market
            .load()?
            .meta()
            .validate_collateral_token(output_token)?;
        let (Some(position), Some(bump)) = (self.position.as_ref(), bump) else {
            return err!(StoreError::PositionIsNotProvided);
        };
        let maybe_initialized = match position.load_init() {
            Ok(mut position) => {
                position.try_init(
                    params.to_position_kind()?,
                    bump,
                    self.store.key(),
                    owner,
                    &self.market.load()?.meta().market_token_mint,
                    output_token,
                )?;
                false
            }
            Err(Error::AnchorError(err)) => {
                if err.error_code_number == ErrorCode::AccountDiscriminatorAlreadySet as u32 {
                    true
                } else {
                    return Err(Error::AnchorError(err));
                }
            }
            Err(err) => {
                return Err(err);
            }
        };
        if maybe_initialized {
            // We need to validate the position if it has been initialized.
            self.validate_position(Some(bump), output_token)?;
        }
        Ok(())
    }

    /// Validate the position to make sure it is initialized and valid.
    fn validate_position(&self, bump: Option<u8>, output_token: &Pubkey) -> Result<()> {
        self.market
            .load()?
            .meta()
            .validate_collateral_token(output_token)?;
        let (Some(position), Some(bump)) = (self.position.as_ref(), bump) else {
            return err!(StoreError::PositionIsNotProvided);
        };
        let position = position.load()?;
        require_eq!(position.bump, bump, StoreError::InvalidPosition);
        Ok(())
    }
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveOrder<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        close = payer,
        constraint = order.fixed.store == store.key() @ StoreError::InvalidOrderToRemove,
        constraint = order.to_account_info().lamports() >= refund @ StoreError::LamportsNotEnough,
        constraint = order.fixed.user == user.key() @ StoreError::UserMismatch,
        seeds = [
            Order::SEED,
            store.key().as_ref(),
            user.key().as_ref(),
            &order.fixed.nonce,
        ],
        bump = order.fixed.bump,
    )]
    pub order: Account<'info, Order>,
    /// CHECK: only used to receive lamports,
    /// and has been checked in `order`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

/// Remove an order.
pub fn remove_order(ctx: Context<RemoveOrder>, refund: u64, reason: &str) -> Result<()> {
    if refund != 0 {
        system_program::transfer(ctx.accounts.transfer_ctx(), refund)?;
    }
    emit_cpi!(RemoveOrderEvent::new(
        ctx.accounts.order.fixed.id,
        ctx.accounts.store.key(),
        ctx.accounts.order.key(),
        ctx.accounts.order.fixed.params.kind,
        ctx.accounts.order.fixed.tokens.market_token,
        ctx.accounts.order.fixed.user,
        reason,
    )?);
    Ok(())
}

impl<'info> internal::Authentication<'info> for RemoveOrder<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> RemoveOrder<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.user.to_account_info(),
            },
        )
    }
}
