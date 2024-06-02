use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{
    constants,
    states::{
        common::{SwapParams, TokenRecord},
        deposit::{Receivers, TokenParams},
        DataStore, Deposit, Market, NonceBytes, Roles, Seed,
    },
    utils::internal,
    DataStoreError,
};

#[derive(Accounts)]
#[instruction(nonce: [u8; 32], tokens_with_feed: Vec<TokenRecord>, swap_params: SwapParams)]
pub struct InitializeDeposit<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        space = 8 + Deposit::init_space(&tokens_with_feed, &swap_params),
        payer = payer,
        seeds = [Deposit::SEED, store.key().as_ref(), payer.key().as_ref(), &nonce],
        bump,
    )]
    pub deposit: Box<Account<'info, Deposit>>,
    #[account(mut, token::authority = payer)]
    pub initial_long_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(mut, token::authority = payer)]
    pub initial_short_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = initial_long_token_account.as_ref().expect("token account not provided").mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            long_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub long_token_deposit_vault: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = initial_short_token_account.as_ref().expect("token account not provided").mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            short_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub short_token_deposit_vault: Option<Box<Account<'info, TokenAccount>>>,
    #[account(has_one = store)]
    pub(crate) market: Box<Account<'info, Market>>,
    #[account(token::authority = payer, token::mint = market.meta.market_token_mint)]
    pub receiver: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Initialize a new [`Deposit`] account.
pub fn initialize_deposit(
    ctx: Context<InitializeDeposit>,
    nonce: NonceBytes,
    tokens_with_feed: Vec<TokenRecord>,
    swap_params: SwapParams,
    token_params: TokenParams,
    ui_fee_receiver: Pubkey,
) -> Result<()> {
    require!(
        token_params.initial_long_token_amount != 0 || token_params.initial_short_token_amount != 0,
        DataStoreError::EmptyDeposit
    );

    if token_params.initial_long_token_amount != 0 {
        anchor_spl::token::transfer(
            ctx.accounts.token_transfer_ctx(true)?,
            token_params.initial_long_token_amount,
        )?;
    }

    if token_params.initial_short_token_amount != 0 {
        anchor_spl::token::transfer(
            ctx.accounts.token_transfer_ctx(false)?,
            token_params.initial_short_token_amount,
        )?;
    }

    ctx.accounts.deposit.init(
        ctx.bumps.deposit,
        ctx.accounts.store.key(),
        &ctx.accounts.market,
        nonce,
        tokens_with_feed,
        ctx.accounts.payer.key(),
        ctx.accounts.initial_long_token_account.as_deref(),
        ctx.accounts.initial_short_token_account.as_deref(),
        Receivers {
            receiver: ctx.accounts.receiver.key(),
            ui_fee_receiver,
        },
        token_params,
        swap_params,
    )
}

impl<'info> internal::Authentication<'info> for InitializeDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

impl<'info> InitializeDeposit<'info> {
    fn token_transfer_ctx(
        &self,
        is_long_token: bool,
    ) -> Result<CpiContext<'_, '_, '_, 'info, token::Transfer<'info>>> {
        let (from, to) = if is_long_token {
            (
                self.initial_long_token_account
                    .as_ref()
                    .ok_or(DataStoreError::MissingDepositTokenAccount)?
                    .to_account_info(),
                self.long_token_deposit_vault
                    .as_ref()
                    .ok_or(DataStoreError::MissingDepositTokenAccount)?
                    .to_account_info(),
            )
        } else {
            (
                self.initial_short_token_account
                    .as_ref()
                    .ok_or(DataStoreError::MissingDepositTokenAccount)?
                    .to_account_info(),
                self.short_token_deposit_vault
                    .as_ref()
                    .ok_or(DataStoreError::MissingDepositTokenAccount)?
                    .to_account_info(),
            )
        };
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            token::Transfer {
                from,
                to,
                authority: self.payer.to_account_info(),
            },
        ))
    }
}

#[derive(Accounts)]
#[instruction(refund: u64)]
pub struct RemoveDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        close = authority,
        constraint = deposit.to_account_info().lamports() >= refund @ DataStoreError::LamportsNotEnough,
        constraint = deposit.fixed.store == store.key() @ DataStoreError::InvalidDepositToRemove,
        constraint = deposit.fixed.senders.user == user.key() @ DataStoreError::UserMismatch,
        seeds = [
            Deposit::SEED,
            store.key().as_ref(),
            user.key().as_ref(),
            &deposit.fixed.nonce,
        ],
        bump = deposit.fixed.bump,
    )]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports,
    /// and has been checked in `deposit`'s constraint.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// The token account for receiving the initial long tokens.
    #[account(mut, token::authority = user)]
    pub initial_long_token: Option<Account<'info, TokenAccount>>,
    /// The token account for receiving the initial short tokens.
    #[account(mut, token::authority = user)]
    pub initial_short_token: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = initial_long_token.as_ref().expect("missing token account").mint,
        constraint = deposit.fixed.tokens.initial_long_token == initial_long_token.as_ref().map(|a| a.mint) @ DataStoreError::InvalidDepositToRemove,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            long_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub long_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = initial_short_token.as_ref().expect("missing token account").mint,
        constraint = deposit.fixed.tokens.initial_short_token == initial_short_token.as_ref().map(|a| a.mint) @ DataStoreError::InvalidDepositToRemove,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            short_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub short_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Remove a deposit.
pub fn remove_deposit(ctx: Context<RemoveDeposit>, refund: u64) -> Result<()> {
    use crate::internal::TransferUtils;

    let transfer = TransferUtils::new(
        ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.store,
        None,
    );
    let deposit = &ctx.accounts.deposit;

    let initial_long_token_amount = deposit.fixed.tokens.params.initial_long_token_amount;
    if initial_long_token_amount != 0 {
        let vault = ctx
            .accounts
            .long_token_deposit_vault
            .as_ref()
            .ok_or(error!(DataStoreError::MissingDepositTokenAccount))?
            .to_account_info();
        let to = ctx
            .accounts
            .initial_long_token
            .as_ref()
            .ok_or(error!(DataStoreError::MissingDepositTokenAccount))?
            .to_account_info();
        transfer.transfer_out(vault, to, initial_long_token_amount)?;
    }

    let initial_short_token_amount = deposit.fixed.tokens.params.initial_short_token_amount;
    if initial_short_token_amount != 0 {
        let vault = ctx
            .accounts
            .short_token_deposit_vault
            .as_ref()
            .ok_or(error!(DataStoreError::MissingDepositTokenAccount))?
            .to_account_info();
        let to = ctx
            .accounts
            .initial_short_token
            .as_ref()
            .ok_or(error!(DataStoreError::MissingDepositTokenAccount))?
            .to_account_info();
        transfer.transfer_out(vault, to, initial_short_token_amount)?;
    }

    system_program::transfer(ctx.accounts.transfer_ctx(), refund)
}

impl<'info> internal::Authentication<'info> for RemoveDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

impl<'info> RemoveDeposit<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.authority.to_account_info(),
                to: self.user.to_account_info(),
            },
        )
    }
}
