use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{approve, Approve, Mint, Token, TokenAccount},
    token_interface,
};
use gmsol_store::{
    cpi::{
        accounts::{CloseOrder, CreateOrder, PrepareUser},
        close_order, create_order, prepare_user,
    },
    ops::order::CreateOrderParams,
    program::GmsolStore,
    states::{common::action::Action, order::OrderKind, NonceBytes, Order},
    utils::{CpiAuthentication, WithStore},
    CoreError,
};

use crate::{
    constants,
    states::{Config, TreasuryConfig},
};

struct SwapOwnerSigner {
    config: Pubkey,
    bump_bytes: [u8; 1],
}

impl SwapOwnerSigner {
    fn as_seeds(&self) -> [&[u8]; 3] {
        [
            constants::SWAP_ORDER_OWNER_SEED,
            self.config.as_ref(),
            &self.bump_bytes,
        ]
    }
}

/// The accounts definition for [`create_swap`](crate::gmsol_treasury::create_swap).
#[derive(Accounts)]
pub struct CreateSwap<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    /// Config.
    #[account(
        has_one = store,
        // Only allow using the authorized treasury config.
        constraint = config.load()?.treasury_config() == Some(&treasury_config.key()) @ CoreError::InvalidArgument,
    )]
    pub config: AccountLoader<'info, Config>,
    /// Treasury Config.
    #[account(
        has_one = config,
        constraint = !treasury_config.load()?.is_deposit_allowed(&swap_in_token.key()).unwrap_or(false) @ CoreError::InvalidArgument,
        constraint = treasury_config.load()?.is_deposit_allowed(&swap_out_token.key())? @ CoreError::InvalidArgument,
    )]
    pub treasury_config: AccountLoader<'info, TreasuryConfig>,
    /// Swap in token.
    pub swap_in_token: Account<'info, Mint>,
    /// Swap out token.
    #[account(constraint = swap_in_token.key() != swap_out_token.key() @ CoreError::InvalidArgument)]
    pub swap_out_token: Account<'info, Mint>,
    /// Swap in token receiver vault.
    #[account(
        mut,
        associated_token::authority = config,
        associated_token::mint = swap_in_token,
    )]
    pub swap_in_token_receiver_vault: Account<'info, TokenAccount>,
    /// Swap out token receiver vault.
    #[account(
        mut,
        associated_token::authority = owner,
        associated_token::mint = swap_out_token,
    )]
    pub swap_out_token_ata: Account<'info, TokenAccount>,
    /// Market.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// Swap order owner.
    #[account(
        mut,
        seeds = [constants::SWAP_ORDER_OWNER_SEED, config.key().as_ref()],
        bump,
    )]
    pub owner: SystemAccount<'info>,
    /// The user account for `owner`.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// The escrow account for swap in token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_in_token_escrow: UncheckedAccount<'info>,
    /// The escrow account for swap out token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_out_token_escrow: UncheckedAccount<'info>,
    /// The order account.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// Store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// Create a swap with the store program.
/// # CHECK
/// Only [`TREASURY_KEEPER`](crate::roles::TREASURY_KEEPER) is allowed to use.
pub(crate) fn unchecked_create_swap<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateSwap<'info>>,
    nonce: NonceBytes,
    swap_path_length: u8,
    swap_in_amount: u64,
    min_swap_out_amount: Option<u64>,
) -> Result<()> {
    ctx.accounts.approve(swap_in_amount)?;

    let signer = SwapOwnerSigner {
        config: ctx.accounts.config.key(),
        bump_bytes: [ctx.bumps.owner],
    };

    // Prepare user.
    let cpi_ctx = ctx.accounts.prepare_user_ctx();
    prepare_user(cpi_ctx.with_signer(&[&signer.as_seeds()]))?;

    // Create order.
    let cpi_ctx = ctx.accounts.create_order_ctx();
    let params = CreateOrderParams {
        kind: OrderKind::MarketSwap,
        decrease_position_swap_type: None,
        execution_lamports: Order::MIN_EXECUTION_LAMPORTS,
        swap_path_length,
        initial_collateral_delta_amount: swap_in_amount,
        size_delta_value: 0,
        is_long: true,
        is_collateral_long: true,
        min_output: min_swap_out_amount.map(u128::from),
        trigger_price: None,
        acceptable_price: None,
    };
    create_order(
        cpi_ctx
            .with_signer(&[&signer.as_seeds()])
            .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        nonce,
        params,
    )?;
    Ok(())
}

impl<'info> WithStore<'info> for CreateSwap<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> CpiAuthentication<'info> for CreateSwap<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        err!(CoreError::PermissionDenied)
    }
}

impl<'info> CreateSwap<'info> {
    fn approve(&self, amount: u64) -> Result<()> {
        require_gt!(amount, 0, CoreError::InvalidArgument);

        let signer = self.config.load()?.signer();
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Approve {
                to: self.swap_in_token_receiver_vault.to_account_info(),
                delegate: self.owner.to_account_info(),
                authority: self.config.to_account_info(),
            },
        );

        approve(ctx.with_signer(&[&signer.as_seeds()]), amount)?;

        Ok(())
    }

    fn prepare_user_ctx(&self) -> CpiContext<'_, '_, '_, 'info, PrepareUser<'info>> {
        CpiContext::new(
            self.store_program.to_account_info(),
            PrepareUser {
                owner: self.owner.to_account_info(),
                store: self.store.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn create_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CreateOrder<'info>> {
        CpiContext::new(
            self.store_program.to_account_info(),
            CreateOrder {
                owner: self.owner.to_account_info(),
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
                user: self.user.to_account_info(),
                order: self.order.to_account_info(),
                position: None,
                initial_collateral_token: Some(self.swap_in_token.to_account_info()),
                final_output_token: self.swap_out_token.to_account_info(),
                long_token: None,
                short_token: None,
                initial_collateral_token_escrow: Some(self.swap_in_token_escrow.to_account_info()),
                final_output_token_escrow: Some(self.swap_out_token_escrow.to_account_info()),
                long_token_escrow: None,
                short_token_escrow: None,
                initial_collateral_token_source: Some(
                    self.swap_in_token_receiver_vault.to_account_info(),
                ),
                final_output_token_ata: Some(self.swap_out_token_ata.to_account_info()),
                long_token_ata: None,
                short_token_ata: None,
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
            },
        )
    }
}

/// The accounts definition for [`cancel_swap`](crate::gmsol_treasury::cancel_swap).
#[derive(Accounts)]
pub struct CancelSwap<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    #[account(
        has_one = store,
    )]
    pub config: AccountLoader<'info, Config>,
    /// Swap order owner.
    #[account(
        mut,
        seeds = [constants::SWAP_ORDER_OWNER_SEED, config.key().as_ref()],
        bump,
    )]
    pub owner: SystemAccount<'info>,
    /// The user account for `owner`.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// Swap in token.
    /// CHECK: check by CPI.
    pub swap_in_token: UncheckedAccount<'info>,
    /// Swap out token.
    /// CHECK: check by CPI.
    pub swap_out_token: UncheckedAccount<'info>,
    /// Swap in token receiver vault.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_in_token_receiver_vault: UncheckedAccount<'info>,
    /// Swap out token receiver vault.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_out_token_ata: UncheckedAccount<'info>,
    /// The escrow account for swap in token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_in_token_escrow: UncheckedAccount<'info>,
    /// The escrow account for swap out token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_out_token_escrow: UncheckedAccount<'info>,
    /// The order account.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// The vault for swap in token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_in_token_vault: UncheckedAccount<'info>,
    /// The vault for swap out token.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub swap_out_token_vault: UncheckedAccount<'info>,
    /// Event authority.
    /// CHECK: check by CPI.
    pub event_authority: UncheckedAccount<'info>,
    /// Store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The token program.
    pub token_program: Program<'info, Token>,
    /// Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// Cancel a swap with the store program.
/// # CHECK
/// Only [`TREASURY_KEEPER`](crate::roles::TREASURY_KEEPER) is allowed to use.
pub(crate) fn unchecked_cancel_swap(ctx: Context<CancelSwap>) -> Result<()> {
    let signer = SwapOwnerSigner {
        config: ctx.accounts.config.key(),
        bump_bytes: [ctx.bumps.owner],
    };
    let cpi_ctx = ctx.accounts.close_order_ctx();
    close_order(
        cpi_ctx.with_signer(&[&signer.as_seeds()]),
        "cancel".to_string(),
    )?;
    Ok(())
}

impl<'info> WithStore<'info> for CancelSwap<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> CpiAuthentication<'info> for CancelSwap<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        err!(CoreError::PermissionDenied)
    }
}

impl<'info> CancelSwap<'info> {
    fn close_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CloseOrder<'info>> {
        CpiContext::new(
            self.store_program.to_account_info(),
            CloseOrder {
                executor: self.owner.to_account_info(),
                store: self.store.to_account_info(),
                owner: self.owner.to_account_info(),
                rent_receiver: self.config.to_account_info(),
                user: self.user.to_account_info(),
                referrer_user: None,
                order: self.order.to_account_info(),
                initial_collateral_token: Some(self.swap_in_token.to_account_info()),
                final_output_token: Some(self.swap_out_token.to_account_info()),
                long_token: None,
                short_token: None,
                initial_collateral_token_escrow: Some(self.swap_in_token_escrow.to_account_info()),
                final_output_token_escrow: Some(self.swap_out_token_escrow.to_account_info()),
                long_token_escrow: None,
                short_token_escrow: None,
                initial_collateral_token_ata: Some(
                    self.swap_in_token_receiver_vault.to_account_info(),
                ),
                final_output_token_ata: Some(self.swap_out_token_ata.to_account_info()),
                long_token_ata: None,
                short_token_ata: None,
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
                event_authority: self.event_authority.to_account_info(),
                program: self.store_program.to_account_info(),
            },
        )
    }
}

/// The accounts definition for [`claim_swapped_tokens`](crate::gmsol_treasury::claim_swapped_tokens).
#[derive(Accounts)]
pub struct ClaimSwappedTokens<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    /// CHECK: check by CPI.
    pub store: UncheckedAccount<'info>,
    #[account(
        has_one = store,
    )]
    pub config: AccountLoader<'info, Config>,
    /// Swap order owner.
    #[account(
        seeds = [constants::SWAP_ORDER_OWNER_SEED, config.key().as_ref()],
        bump,
    )]
    pub owner: SystemAccount<'info>,
    /// Token.
    pub token: InterfaceAccount<'info, token_interface::Mint>,
    /// Swap out token receiver vault.
    #[account(
        mut,
        associated_token::authority = owner,
        associated_token::mint = token,
    )]
    pub swap_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    /// Swap in token receiver vault.
    #[account(
        mut,
        associated_token::authority = config,
        associated_token::mint = token,
    )]
    pub receiver_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    /// Store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The token program.
    pub token_program: Interface<'info, token_interface::TokenInterface>,
}

impl<'info> WithStore<'info> for ClaimSwappedTokens<'info> {
    fn store_program(&self) -> AccountInfo<'info> {
        self.store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }
}

impl<'info> CpiAuthentication<'info> for ClaimSwappedTokens<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        err!(CoreError::PermissionDenied)
    }
}

/// Claim swapped tokens.
/// # CHECK
/// Only [`TREASURY_KEEPER`](crate::roles::TREASURY_KEEPER) can use.
pub(crate) fn unchecked_claim_swapped_tokens(ctx: Context<ClaimSwappedTokens>) -> Result<()> {
    let amount = ctx.accounts.swap_vault.amount;

    if amount == 0 {
        msg!("[Treasury] empty swap vault");
        return Ok(());
    }

    let decimals = ctx.accounts.token.decimals;

    let signer = SwapOwnerSigner {
        config: ctx.accounts.config.key(),
        bump_bytes: [ctx.bumps.owner],
    };

    let ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token_interface::TransferChecked {
            from: ctx.accounts.swap_vault.to_account_info(),
            mint: ctx.accounts.token.to_account_info(),
            to: ctx.accounts.receiver_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    token_interface::transfer_checked(ctx.with_signer(&[&signer.as_seeds()]), amount, decimals)?;
    Ok(())
}
