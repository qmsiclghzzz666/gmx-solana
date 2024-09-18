use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::{
    ops::{
        execution_fee::TransferExecutionFeeOps,
        order::{CreateOrderOps, CreateOrderParams},
    },
    states::{
        order::{OrderKind, OrderV2},
        Market, NonceBytes, Position, Seed, Store,
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
        _ => {
            todo!()
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
