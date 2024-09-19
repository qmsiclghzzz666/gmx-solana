use std::collections::HashSet;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    events::RemoveOrderEvent,
    ops::{
        execution_fee::TransferExecutionFeeOps,
        order::{CreateOrderOps, CreateOrderParams},
    },
    states::{
        order::{OrderKind, OrderV2},
        Market, NonceBytes, Position, RoleKey, Seed, Store,
    },
    utils::{
        internal::{self, Authentication},
        token::is_associated_token_account,
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
    pub initial_collateral_token_escrow: Box<Account<'info, TokenAccount>>,
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
        init_if_needed,
        payer = owner,
        space = 8 + Position::INIT_SPACE,
        seeds = [
            Position::SEED,
            store.key().as_ref(),
            market.load()?.meta().market_token_mint.as_ref(),
            params.collateral_token(market.load()?.meta()).as_ref(),
            &[params.to_position_kind()? as u8],
        ],
        bump,
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

pub(crate) fn create_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
    nonce: NonceBytes,
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
        .nonce(&nonce)
        .bump(ctx.bumps.order)
        .params(params)
        .swap_path(ctx.remaining_accounts)
        .build();

    match params.kind {
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
            let position = accounts
                .position
                .as_ref()
                .ok_or(error!(CoreError::PositionIsRequired))?;
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
            let position_bump = ctx
                .bumps
                .position
                .as_ref()
                .ok_or(error!(CoreError::PositionIsRequired))?;
            ops.increase()
                .position(position.clone())
                .position_bump(*position_bump)
                .initial_collateral_token(initial_collateral.as_ref())
                .long_token(long_token.as_ref())
                .short_token(short_token.as_ref())
                .build()
                .execute()?;
        }
        OrderKind::MarketDecrease | OrderKind::LimitDecrease | OrderKind::StopLossDecrease => {
            let position = accounts
                .position
                .as_ref()
                .ok_or(error!(CoreError::PositionIsRequired))?;
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
            let position_bump = ctx
                .bumps
                .position
                .as_ref()
                .ok_or(error!(CoreError::PositionIsRequired))?;
            ops.decrease()
                .position(position.clone())
                .position_bump(*position_bump)
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

    fn transfer_tokens(&self, params: &CreateOrderParams) -> Result<()> {
        if !matches!(
            params.kind,
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
                .as_ref()
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
    pub store: AccountLoader<'info, Store>,
    /// THe owner of the order.
    /// CHECK: only used to validate and receive fund.
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
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
    /// The system program.
    pub system_program: Program<'info, System>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// The associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn close_order(ctx: Context<CloseOrder>, reason: &str) -> Result<()> {
    let accounts = &ctx.accounts;
    let should_continue_when_atas_are_missing = accounts.preprocess()?;
    if accounts.transfer_to_atas(should_continue_when_atas_are_missing)? {
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

    fn close(&self) -> Result<()> {
        self.order.close(self.owner.to_account_info())?;
        Ok(())
    }
}
