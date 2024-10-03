use std::collections::HashSet;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
    token_interface,
};
use gmsol_model::utils::apply_factor;

use crate::{
    constants,
    events::RemoveOrderEvent,
    ops::{
        execution_fee::TransferExecutionFeeOps,
        order::{CreateOrderOps, CreateOrderParams},
    },
    states::{
        order::{OrderKind, OrderV2},
        position::PositionKind,
        user::UserHeader,
        FactorKey, Market, NonceBytes, Position, RoleKey, Seed, Store,
    },
    utils::{
        internal::{self, Authentication},
        token::{
            is_associated_token_account, is_associated_token_account_with_program_id,
            must_be_uninitialized,
        },
    },
    CoreError,
};

/// The accounts definitions for the `prepare_swap_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareSwapOrderEscrow<'info> {
    /// The owner of the order.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The order owning these escrow accounts.
    /// CHECK: The order account don't have to be initialized.
    #[account(
        seeds = [OrderV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub order: UncheckedAccount<'info>,
    /// Swap in token (will be stored as initial collateral token in order account).
    pub swap_in_token: Box<Account<'info, Mint>>,
    /// Swap out token (will be stored as collateral/output token in order account).
    pub swap_out_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving the swap in tokens from the owner.
    /// It will be stored as initial collateral token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = swap_in_token,
        associated_token::authority = order,
    )]
    pub swap_in_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the swap out tokens after the swap.
    /// It will be stored as final output token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = swap_out_token,
        associated_token::authority = order,
    )]
    pub swap_out_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_swap_order_escrow(
    _ctx: Context<PrepareSwapOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definitions for the `prepare_increase_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareIncreaseOrderEscrow<'info> {
    /// The owner of the order.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The order owning these escrow accounts.
    /// CHECK: The order account don't have to be initialized.
    #[account(
        seeds = [OrderV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub order: UncheckedAccount<'info>,
    /// Initial collateral token (will be stored as initial collateral token in order account).
    pub initial_collateral_token: Box<Account<'info, Mint>>,
    /// Long token of the market.
    pub long_token: Box<Account<'info, Mint>>,
    /// Short token of the market.
    pub short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving the initial collateral tokens from the owner.
    /// It will be stored as initial collateral token account in order account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = initial_collateral_token,
        associated_token::authority = order,
    )]
    pub initial_collateral_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in long tokens after increasing position.
    /// It will be stored as long token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in short tokens after increasing position.
    /// It will be stored as short token account.
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_increase_order_escrow(
    _ctx: Context<PrepareIncreaseOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// The accounts definitions for the `prepare_decrease_order_escrow` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32])]
pub struct PrepareDecreaseOrderEscrow<'info> {
    /// The payer of this instruction.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// The owner of the order.
    /// CHECK: only used as an identifier.
    pub owner: UncheckedAccount<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// The order owning these escrow accounts.
    /// CHECK: The order account don't have to be initialized.
    #[account(
        seeds = [OrderV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub order: UncheckedAccount<'info>,
    /// Final output token (will be stored as final output token in order account).
    pub final_output_token: Box<Account<'info, Mint>>,
    /// Long token of the market.
    pub long_token: Box<Account<'info, Mint>>,
    /// Short token of the market.
    pub short_token: Box<Account<'info, Mint>>,
    /// The escrow account for receving final output tokens after decreasing position.
    /// It will be stored as final output token account in order account.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = final_output_token,
        associated_token::authority = order,
    )]
    pub final_output_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in long tokens or pnl tokens after decreasing position.
    /// It will be stored as long token account.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The escrow account for receving the funding rebate in short tokens or pnl tokens after decreasing position.
    /// It will be stored as short token account.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Box<Account<'info, TokenAccount>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn prepare_decrease_order_escrow(
    _ctx: Context<PrepareDecreaseOrderEscrow>,
    _nonce: NonceBytes,
) -> Result<()> {
    Ok(())
}

/// Prepare position.
#[derive(Accounts)]
#[instruction(params: CreateOrderParams)]
pub struct PreparePosition<'info> {
    /// The owner of the order to be created.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Market.
    #[account(has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// The position.
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Position::INIT_SPACE,
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            owner.key().as_ref(),
            market.load()?.meta().market_token_mint.as_ref(),
            params.collateral_token(market.load()?.meta()).as_ref(),
            &[params.to_position_kind()? as u8],
        ],
        bump,
    )]
    pub position: AccountLoader<'info, Position>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

pub(crate) fn prepare_position(
    ctx: Context<PreparePosition>,
    params: &CreateOrderParams,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let meta = *ctx.accounts.market.load()?.meta();
    let market_token = meta.market_token_mint;
    let collateral_token = params.collateral_token(&meta);
    validate_and_initialize_position_if_needed(
        &ctx.accounts.position,
        ctx.bumps.position,
        params.to_position_kind()?,
        &ctx.accounts.owner,
        collateral_token,
        &market_token,
        &store,
        ctx.accounts.system_program.to_account_info(),
    )?;
    Ok(())
}

fn validate_and_initialize_position_if_needed<'info>(
    position: &AccountLoader<'info, Position>,
    bump: u8,
    kind: PositionKind,
    owner: &AccountInfo<'info>,
    collateral_token: &Pubkey,
    market_token: &Pubkey,
    store: &Pubkey,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let mut should_transfer_in = false;

    let owner_key = owner.key;
    match position.load_init() {
        Ok(mut position) => {
            position.try_init(
                kind,
                bump,
                *store,
                owner_key,
                market_token,
                collateral_token,
            )?;
            should_transfer_in = true;
        }
        Err(Error::AnchorError(err)) => {
            if err.error_code_number != ErrorCode::AccountDiscriminatorAlreadySet as u32 {
                return Err(Error::AnchorError(err));
            }
        }
        Err(err) => {
            return Err(err);
        }
    }
    position.exit(&crate::ID)?;
    validate_position(
        &*position.load()?,
        bump,
        kind,
        owner_key,
        collateral_token,
        market_token,
        store,
    )?;

    if should_transfer_in {
        TransferExecutionFeeOps::builder()
            .payment(position.to_account_info())
            .payer(owner.clone())
            .execution_lamports(OrderV2::position_cut_rent()?)
            .system_program(system_program)
            .build()
            .execute()?;
    }
    Ok(())
}

fn validate_position(
    position: &Position,
    bump: u8,
    kind: PositionKind,
    owner: &Pubkey,
    collateral_token: &Pubkey,
    market_token: &Pubkey,
    store: &Pubkey,
) -> Result<()> {
    require_eq!(position.bump, bump, CoreError::InvalidPosition);
    require_eq!(position.kind()?, kind, CoreError::InvalidPosition);
    require_eq!(position.owner, *owner, CoreError::InvalidPosition);
    require_eq!(
        position.collateral_token,
        *collateral_token,
        CoreError::InvalidPosition
    );
    require_eq!(
        position.market_token,
        *market_token,
        CoreError::InvalidPosition
    );
    require_eq!(position.store, *store, CoreError::InvalidPosition);
    Ok(())
}

/// The accounts definitions for `create_order` instruction.
#[derive(Accounts)]
#[instruction(nonce: [u8; 32], params: CreateOrderParams)]
pub struct CreateOrder<'info> {
    /// The owner of the order to be created.
    #[account(mut)]
    pub owner: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Market.
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    /// User Account.
    #[account(
        mut,
        constraint = user.load()?.is_initialized() @ CoreError::InvalidUserAccount,
        has_one = owner,
        has_one = store,
        seeds = [UserHeader::SEED, store.key().as_ref(), owner.key().as_ref()],
        bump = user.load()?.bump,
    )]
    pub user: AccountLoader<'info, UserHeader>,
    /// The order to be created.
    #[account(
        init,
        space = 8 + OrderV2::INIT_SPACE,
        payer = owner,
        seeds = [OrderV2::SEED, store.key().as_ref(), owner.key().as_ref(), &nonce],
        bump,
    )]
    pub order: AccountLoader<'info, OrderV2>,
    /// The related position.
    #[account(
        mut,
        has_one = store,
        has_one = owner,
        constraint = position.load()?.market_token == market.load()?.meta().market_token_mint @ CoreError::MarketTokenMintMismatched,
        constraint = position.load()?.collateral_token == *params.collateral_token(&*market.load()?) @ CoreError::InvalidPosition,
        constraint = position.load()?.kind()? == params.to_position_kind()? @ CoreError::InvalidPosition,
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            owner.key().as_ref(),
            market.load()?.meta().market_token_mint.as_ref(),
            params.collateral_token(market.load()?.meta()).as_ref(),
            &[params.to_position_kind()? as u8],
        ],
        bump = position.load()?.bump,
    )]
    pub position: Option<AccountLoader<'info, Position>>,
    /// Initial collateral token / swap in token.
    /// Only required by increase and swap orders.
    pub initial_collateral_token: Option<Box<Account<'info, Mint>>>,
    /// Final output token.
    /// Used as collateral token / swap out token for increase and swap orders;
    /// and used as final output token for decrease orders.
    ///
    /// For the case of increase or swap orders, it will be checked to be a valid
    /// collateral / swap out token.
    pub final_output_token: Box<Account<'info, Mint>>,
    /// Long token of the market.
    #[account(constraint = market.load()?.meta().long_token_mint == long_token.key())]
    pub long_token: Option<Box<Account<'info, Mint>>>,
    /// Short token of the market.
    #[account(constraint = market.load()?.meta().short_token_mint == short_token.key())]
    pub short_token: Option<Box<Account<'info, Mint>>>,
    /// Initial collateral token escrow account.
    /// Only requried by increase and swap orders.
    #[account(
        mut,
        associated_token::mint = initial_collateral_token,
        associated_token::authority = order,
    )]
    pub initial_collateral_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// Final output token escrow account.
    /// Only required by decrease and swap orders.
    #[account(
        mut,
        associated_token::mint = final_output_token,
        associated_token::authority = order,
    )]
    pub final_output_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// Long token escrow.
    /// Only required by increase and decrease orders.
    #[account(
        mut,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// Short token escrow.
    /// Only required by increase and decrease orders.
    #[account(
        mut,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The source initial token account.
    /// Only requried by increase and swap orders.
    #[account(
        mut,
        token::mint = initial_collateral_token,
    )]
    pub initial_collateral_token_source: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA for receiving the final output tokens.
    /// Only required by decrease and swap orders.
    #[account(
        associated_token::mint = final_output_token,
        associated_token::authority = owner
    )]
    pub final_output_token_ata: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA for receiving the long tokens.
    /// Only required by increase and decrease orders.
    #[account(
        associated_token::mint = long_token,
        associated_token::authority = owner
    )]
    pub long_token_ata: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA for receiving the long tokens.
    /// Only required by increase and decrease orders.
    #[account(
        associated_token::mint = short_token,
        associated_token::authority = owner
    )]
    pub short_token_ata: Option<Box<Account<'info, TokenAccount>>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[inline(never)]
pub(crate) fn create_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
    nonce: &NonceBytes,
    params: &CreateOrderParams,
) -> Result<()> {
    let accounts = ctx.accounts;
    accounts.transfer_execution_fee(params)?;
    accounts.transfer_tokens(params)?;

    let ops = CreateOrderOps::builder()
        .order(accounts.order.clone())
        .market(accounts.market.clone())
        .store(accounts.store.clone())
        .owner(accounts.owner.to_account_info())
        .nonce(nonce)
        .bump(ctx.bumps.order)
        .params(params)
        .swap_path(ctx.remaining_accounts)
        .build();

    let kind = params.kind;
    match kind {
        OrderKind::MarketSwap | OrderKind::LimitSwap => {
            let swap_in = accounts
                .initial_collateral_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let swap_out = accounts
                .final_output_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            ops.swap()
                .swap_in_token(swap_in.as_ref())
                .swap_out_token(swap_out.as_ref())
                .build()
                .execute()?;
        }
        OrderKind::MarketIncrease | OrderKind::LimitIncrease => {
            let initial_collateral = accounts
                .initial_collateral_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let long_token = accounts
                .long_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let short_token = accounts
                .short_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            ops.increase()
                .position(
                    accounts
                        .position
                        .as_ref()
                        .ok_or(error!(CoreError::PositionIsRequired))?,
                )
                .initial_collateral_token(initial_collateral.as_ref())
                .long_token(long_token.as_ref())
                .short_token(short_token.as_ref())
                .build()
                .execute()?;
        }
        OrderKind::MarketDecrease | OrderKind::LimitDecrease | OrderKind::StopLossDecrease => {
            let final_output = accounts
                .final_output_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let long_token = accounts
                .long_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let short_token = accounts
                .short_token_escrow
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            ops.decrease()
                .position(
                    accounts
                        .position
                        .as_ref()
                        .ok_or(error!(CoreError::PositionIsRequired))?,
                )
                .final_output_token(final_output.as_ref())
                .long_token(long_token.as_ref())
                .short_token(short_token.as_ref())
                .build()
                .execute()?;
        }
        _ => {
            return err!(CoreError::OrderKindNotAllowed);
        }
    }

    Ok(())
}

impl<'info> CreateOrder<'info> {
    fn transfer_execution_fee(&self, params: &CreateOrderParams) -> Result<()> {
        TransferExecutionFeeOps::builder()
            .payment(self.order.to_account_info())
            .payer(self.owner.to_account_info())
            .execution_lamports(params.execution_fee)
            .system_program(self.system_program.to_account_info())
            .build()
            .execute()
    }

    fn transfer_tokens(&mut self, params: &CreateOrderParams) -> Result<()> {
        let kind = params.kind;
        if !matches!(
            kind,
            OrderKind::MarketSwap
                | OrderKind::LimitSwap
                | OrderKind::MarketIncrease
                | OrderKind::LimitIncrease
        ) {
            return Ok(());
        }
        let amount = params.initial_collateral_delta_amount;
        if amount != 0 {
            let token = self
                .initial_collateral_token
                .as_ref()
                .ok_or(error!(CoreError::MissingInitialCollateralToken))?;
            let from = self
                .initial_collateral_token_source
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let to = self
                .initial_collateral_token_escrow
                .as_mut()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;

            transfer_checked(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    TransferChecked {
                        from: from.to_account_info(),
                        mint: token.to_account_info(),
                        to: to.to_account_info(),
                        authority: self.owner.to_account_info(),
                    },
                ),
                amount,
                token.decimals,
            )?;

            to.reload()?;
        }
        Ok(())
    }
}

/// The accounts definition for the `close_order instruction.
#[event_cpi]
#[derive(Accounts)]
pub struct CloseOrder<'info> {
    /// The executor of this instruction.
    pub executor: Signer<'info>,
    /// The store.
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
    /// THe owner of the order.
    /// CHECK: only used to validate and receive fund.
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    /// User Account.
    #[account(
        mut,
        constraint = user.load()?.is_initialized() @ CoreError::InvalidUserAccount,
        has_one = owner,
        has_one = store,
        seeds = [UserHeader::SEED, store.key().as_ref(), owner.key().as_ref()],
        bump = user.load()?.bump,
    )]
    pub user: AccountLoader<'info, UserHeader>,
    /// Referrer User Account.
    #[account(
        mut,
        constraint = referrer_user.key() != user.key() @ CoreError::InvalidArgument,
        constraint = referrer_user.load()?.is_initialized() @ CoreError::InvalidUserAccount,
        constraint = referrer_user.load()?.owner == *user.load()?.referral().referrer().ok_or(CoreError::InvalidArgument)? @ CoreError::InvalidArgument,
        has_one = store,
        seeds = [UserHeader::SEED, store.key().as_ref(), user.load()?.referral().referrer().ok_or(CoreError::InvalidArgument)?.as_ref()],
        bump = referrer_user.load()?.bump,
    )]
    pub referrer_user: Option<AccountLoader<'info, UserHeader>>,
    /// Order to close.
    #[account(
        mut,
        constraint = order.load()?.header.owner == owner.key() @ CoreError::OwnerMismatched,
        constraint = order.load()?.header.store == store.key() @ CoreError::StoreMismatched,
        constraint = order.load()?.tokens.initial_collateral.account() == initial_collateral_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = order.load()?.tokens.final_output_token.account() == final_output_token_escrow.as_ref().map(|a| a.key()) @ CoreError::TokenAccountMismatched,
        constraint = order.load()?.tokens.long_token.account() == long_token_escrow.as_ref().map(|a| a.key())@ CoreError::TokenAccountMismatched,
        constraint = order.load()?.tokens.short_token.account() == short_token_escrow.as_ref().map(|a| a.key())@ CoreError::TokenAccountMismatched,
    )]
    pub order: AccountLoader<'info, OrderV2>,
    /// Initial collateral token.
    pub initial_collateral_token: Option<Box<Account<'info, Mint>>>,
    /// Final output token.
    pub final_output_token: Option<Box<Account<'info, Mint>>>,
    /// Long token.
    pub long_token: Option<Box<Account<'info, Mint>>>,
    /// Short token.
    pub short_token: Option<Box<Account<'info, Mint>>>,
    /// GT mint.
    #[account(
        mut,
        mint::authority = store,
        seeds = [
            constants::GT_MINT_SEED,
            store.key().as_ref(),
        ],
        bump,
        owner = gt_token_program.key(),
    )]
    pub gt_mint: InterfaceAccount<'info, token_interface::Mint>,
    /// The escrow account for initial collateral tokens.
    #[account(
        mut,
        associated_token::mint = initial_collateral_token,
        associated_token::authority = order,
    )]
    pub initial_collateral_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for final output tokens.
    #[account(
        mut,
        associated_token::mint = final_output_token,
        associated_token::authority = order,
    )]
    pub final_output_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for long tokens.
    #[account(
        mut,
        associated_token::mint = long_token,
        associated_token::authority = order,
    )]
    pub long_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The escrow account for short tokens.
    #[account(
        mut,
        associated_token::mint = short_token,
        associated_token::authority = order,
    )]
    pub short_token_escrow: Option<Box<Account<'info, TokenAccount>>>,
    /// The ATA for initial collateral token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(initial_collateral_token_ata.key, owner.key, &initial_collateral_token.as_ref().map(|a| a.key()).expect("must provide")) @ CoreError::NotAnATA,
    )]
    pub initial_collateral_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for final output token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(final_output_token_ata.key, owner.key, &final_output_token.as_ref().map(|a| a.key()).expect("must provide")) @ CoreError::NotAnATA,
    )]
    pub final_output_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for long token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(long_token_ata.key, owner.key, &long_token.as_ref().map(|a| a.key()).expect("must provide")) @ CoreError::NotAnATA,
    )]
    pub long_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for initial collateral token of owner.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account(short_token_ata.key, owner.key, &short_token.as_ref().map(|a| a.key()).expect("must provide")) @ CoreError::NotAnATA,
    )]
    pub short_token_ata: Option<UncheckedAccount<'info>>,
    /// The ATA for GT.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account_with_program_id(gt_ata.key, owner.key, &gt_mint.key(), &gt_token_program.key()) @ CoreError::NotAnATA,
    )]
    pub gt_ata: UncheckedAccount<'info>,
    /// The ATA for GT.
    /// CHECK: should be checked during the execution.
    #[account(
        mut,
        constraint = is_associated_token_account_with_program_id(gt_ata_for_referrer.key, &referrer_user.as_ref().expect("must provided").load()?.owner, &gt_mint.key(), &gt_token_program.key()) @ CoreError::NotAnATA,
    )]
    pub gt_ata_for_referrer: Option<UncheckedAccount<'info>>,
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The token program for GT.
    pub gt_token_program: Interface<'info, token_interface::TokenInterface>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn close_order(ctx: Context<CloseOrder>, reason: &str) -> Result<()> {
    let accounts = &ctx.accounts;
    let should_continue_when_atas_are_missing = accounts.preprocess()?;
    let transfer_success = accounts.transfer_to_atas(should_continue_when_atas_are_missing)?;
    let mint_success = accounts.mint_gt_reward(should_continue_when_atas_are_missing)?;
    if transfer_success && mint_success {
        {
            let order_address = accounts.order.key();
            let order = accounts.order.load()?;
            emit_cpi!(RemoveOrderEvent::new(
                order.header.id,
                order.header.store,
                order_address,
                order.params.kind()?,
                order.market_token,
                order.header.owner,
                reason
            )?);
        }
        accounts.close()?;
    } else {
        msg!("Some ATAs are not initialized, skip the close");
    }
    Ok(())
}

impl<'info> internal::Authentication<'info> for CloseOrder<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.executor
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

type ShouldContinueWhenATAsAreMissing = bool;
type Success = bool;

impl<'info> CloseOrder<'info> {
    fn preprocess(&self) -> Result<ShouldContinueWhenATAsAreMissing> {
        if self.executor.key == self.owner.key {
            Ok(true)
        } else {
            self.only_role(RoleKey::ORDER_KEEPER)?;
            {
                let order = self.order.load()?;
                if order.header.action_state()?.is_completed_or_cancelled() {
                    Ok(false)
                } else {
                    err!(CoreError::PermissionDenied)
                }
            }
        }
    }

    fn transfer_to_atas(&self, init_if_needed: bool) -> Result<Success> {
        use crate::utils::token::TransferAllFromEscrowToATA;

        let signer = self.order.load()?.signer();
        let seeds = signer.as_seeds();

        let mut seen = HashSet::<_>::default();

        let builder = TransferAllFromEscrowToATA::builder()
            .system_program(self.system_program.to_account_info())
            .token_program(self.token_program.to_account_info())
            .associated_token_program(self.associated_token_program.to_account_info())
            .payer(self.executor.to_account_info())
            .owner(self.owner.to_account_info())
            .escrow_authority(self.order.to_account_info())
            .seeds(&seeds)
            .init_if_needed(init_if_needed);

        for (escrow, ata, token) in [
            (
                self.initial_collateral_token_escrow.as_ref(),
                self.initial_collateral_token_ata.as_ref(),
                self.initial_collateral_token.as_ref(),
            ),
            (
                self.final_output_token_escrow.as_ref(),
                self.final_output_token_ata.as_ref(),
                self.final_output_token.as_ref(),
            ),
            (
                self.long_token_escrow.as_ref(),
                self.long_token_ata.as_ref(),
                self.long_token.as_ref(),
            ),
            (
                self.short_token_escrow.as_ref(),
                self.short_token_ata.as_ref(),
                self.short_token.as_ref(),
            ),
        ] {
            if let Some(escrow) = escrow {
                if !seen.insert(escrow.key()) {
                    continue;
                }
                let ata = ata.ok_or(error!(CoreError::TokenAccountNotProvided))?;
                let token = token.ok_or(error!(CoreError::TokenMintNotProvided))?;

                if !builder
                    .clone()
                    .mint(token.to_account_info())
                    .ata(ata.to_account_info())
                    .escrow(escrow)
                    .build()
                    .execute()?
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn mint_gt_reward(&self, init_if_needed: bool) -> Result<Success> {
        use anchor_spl::{
            associated_token::{create, Create},
            token::accessor::amount as access_amount,
            token_2022::{mint_to, MintTo},
        };

        let amount = self.order.load()?.gt_reward;
        if amount != 0 {
            {
                let ata = &self.gt_ata;
                let mint = &self.gt_mint;

                if must_be_uninitialized(ata) {
                    if !init_if_needed {
                        return Ok(false);
                    }
                    create(CpiContext::new(
                        self.associated_token_program.to_account_info(),
                        Create {
                            payer: self.executor.to_account_info(),
                            associated_token: ata.to_account_info(),
                            authority: self.owner.to_account_info(),
                            mint: mint.to_account_info(),
                            system_program: self.system_program.to_account_info(),
                            token_program: self.gt_token_program.to_account_info(),
                        },
                    ))?;
                }

                // TODO: unpack the ATA to ensure it is a valid token account.

                let ctx = CpiContext::new(
                    self.gt_token_program.to_account_info(),
                    MintTo {
                        mint: mint.to_account_info(),
                        to: ata.to_account_info(),
                        authority: self.store.to_account_info(),
                    },
                );
                mint_to(ctx.with_signer(&[&self.store.load()?.pda_seeds()]), amount)?;

                msg!("[GT] minted {} units of GT", amount);

                // Update the rank of the user.
                {
                    let total_amount = access_amount(ata)?;
                    msg!("[GT] updating rank with total amount: {}", total_amount);
                    self.user
                        .load_mut()?
                        .gt
                        .update_rank(&*self.store.load()?, total_amount);
                }

                // Make sure the mint can only be done once.
                self.order.load_mut()?.gt_reward = 0;
            }
            self.mint_gt_reward_for_referrer(amount)?;
        }

        Ok(true)
    }

    fn mint_gt_reward_for_referrer(&self, amount: u64) -> Result<()> {
        use anchor_spl::{
            token::accessor::amount as access_amount,
            token_2022::{mint_to, MintTo},
        };

        // Mint referral reward for the referrer.
        let Some(referrer) = self.user.load()?.referral().referrer().copied() else {
            return Ok(());
        };

        let referrer_user = self
            .referrer_user
            .as_ref()
            .ok_or(error!(CoreError::InvalidArgument))?;
        require_eq!(
            referrer_user.load()?.owner,
            referrer,
            CoreError::InvalidArgument
        );

        let factor = *self
            .store
            .load()?
            .get_factor_by_key(FactorKey::GTReferralReward);
        let reward: u64 =
            apply_factor::<_, { constants::MARKET_DECIMALS }>(&(amount as u128), &factor)
                .ok_or(error!(CoreError::InvalidGTConfig))?
                .try_into()
                .map_err(|_| error!(CoreError::TokenAmountOverflow))?;

        if reward != 0 {
            let ata = self
                .gt_ata_for_referrer
                .as_ref()
                .ok_or(error!(CoreError::TokenAccountNotProvided))?;
            let mint = &self.gt_mint;

            if must_be_uninitialized(ata) {
                msg!(
                    "[GT] referrer reward has been cancelled because the ATA account is not found."
                );
                return Ok(());
            }

            // TODO: unpack the ATA to ensure it is a valid token account.

            {
                let mut store = self.store.load_mut()?;
                let mut referrer_user = referrer_user.load_mut()?;

                store.gt_mut().record_minted(reward)?;
                referrer_user.gt.minted = referrer_user
                    .gt
                    .minted
                    .checked_add(reward)
                    .ok_or(error!(CoreError::TokenAmountOverflow))?;
                referrer_user.gt.last_minted_at = store.gt().last_minted_at;
            }

            let ctx = CpiContext::new(
                self.gt_token_program.to_account_info(),
                MintTo {
                    mint: mint.to_account_info(),
                    to: ata.to_account_info(),
                    authority: self.store.to_account_info(),
                },
            );
            mint_to(ctx.with_signer(&[&self.store.load()?.pda_seeds()]), reward)?;

            msg!("[GT] minted {} units of GT to the referrer", reward);

            // Update the rank of the referrer.
            {
                let total_amount = access_amount(ata)?;
                msg!("[GT] updating rank with total amount: {}", total_amount);
                referrer_user
                    .load_mut()?
                    .gt
                    .update_rank(&*self.store.load()?, total_amount);
            }
        }

        Ok(())
    }

    fn close(&self) -> Result<()> {
        self.order.close(self.owner.to_account_info())?;
        Ok(())
    }
}
