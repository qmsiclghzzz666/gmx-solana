use std::collections::BTreeSet;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{self, Token, TokenAccount};
use data_store::{
    cpi::accounts::{GetMarketMeta, GetTokenConfig, InitializeOrder},
    program::DataStore,
    states::{
        common::SwapParams,
        order::{OrderKind, OrderParams},
        NonceBytes,
    },
};

use crate::{
    events::OrderCreatedEvent,
    utils::{market::get_and_validate_swap_path, ControllerSeeds},
    ExchangeError,
};

#[derive(Accounts)]
pub struct CreateOrder<'info> {
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_controller: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: only used to invoke CPI and then checked and initilized by it.
    #[account(mut)]
    pub order: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub position: Option<UncheckedAccount<'info>>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub market: UncheckedAccount<'info>,
    #[account(mut)]
    pub initial_collateral_token_account: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK: check by CPI.
    pub final_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK: check by CPI.
    pub secondary_output_token_account: Option<Box<Account<'info, TokenAccount>>>,
    #[account(
        mut,
        token::mint = initial_collateral_token_account.as_ref().expect("sender must be provided").mint,
        seeds = [
            data_store::constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            initial_collateral_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub initial_collateral_token_vault: Option<Box<Account<'info, TokenAccount>>>,
    pub data_store_program: Program<'info, DataStore>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// Create Order.
pub fn create_order<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
    nonce: NonceBytes,
    params: CreateOrderParams,
) -> Result<()> {
    let order = &params.order;
    let controller = ControllerSeeds::new(ctx.accounts.store.key, ctx.bumps.authority);

    let (tokens, swap) = match &order.kind {
        OrderKind::MarketIncrease => {
            if order.initial_collateral_delta_amount != 0 {
                anchor_spl::token::transfer(
                    ctx.accounts.token_transfer_ctx()?,
                    order.initial_collateral_delta_amount,
                )?;
            }
            ctx.accounts.handle_tokens_for_increase_order(
                &params.output_token,
                ctx.remaining_accounts,
                params.swap_length as usize,
            )?
        }
        OrderKind::MarketDecrease | OrderKind::Liquidation => {
            ctx.accounts.handle_tokens_for_decrease_order(
                &params.output_token,
                ctx.remaining_accounts,
                params.swap_length as usize,
            )?
        }
        _ => {
            return err!(ExchangeError::UnsupportedOrderKind);
        }
    };

    data_store::cpi::initialize_order(
        ctx.accounts
            .initialize_order_ctx()
            .with_signer(&[&controller.as_seeds()]),
        nonce,
        ctx.accounts.to_tokens_with_feed(tokens)?,
        swap,
        order.clone(),
        params.output_token,
        params.ui_fee_receiver,
    )?;

    require_gte!(
        ctx.accounts.order.lamports() + params.execution_fee,
        super::MAX_ORDER_EXECUTION_FEE,
        ExchangeError::NotEnoughExecutionFee
    );
    if params.execution_fee != 0 {
        system_program::transfer(ctx.accounts.transfer_ctx(), params.execution_fee)?;
    }

    emit!(OrderCreatedEvent {
        ts: Clock::get()?.unix_timestamp,
        store: ctx.accounts.store.key(),
        order: ctx.accounts.order.key(),
        position: ctx.accounts.position.as_ref().map(|a| a.key()),
    });
    Ok(())
}

/// Create Order Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateOrderParams {
    /// Order Params.
    pub order: OrderParams,
    /// Swap out token or collateral token.
    pub output_token: Pubkey,
    /// Ui fee receiver.
    pub ui_fee_receiver: Pubkey,
    /// Execution fee.
    pub execution_fee: u64,
    /// Swap path length.
    pub swap_length: u8,
}

impl<'info> CreateOrder<'info> {
    fn initialize_order_ctx(&self) -> CpiContext<'_, '_, '_, 'info, InitializeOrder<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            InitializeOrder {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                payer: self.payer.to_account_info(),
                order: self.order.to_account_info(),
                position: self.position.as_ref().map(|a| a.to_account_info()),
                market: self.market.to_account_info(),
                initial_collateral_token_account: self
                    .initial_collateral_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                final_output_token_account: self
                    .final_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                secondary_output_token_account: self
                    .secondary_output_token_account
                    .as_ref()
                    .map(|a| a.to_account_info()),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn token_transfer_ctx(&self) -> Result<CpiContext<'_, '_, '_, 'info, token::Transfer<'info>>> {
        let (Some(from), Some(to)) = (
            self.initial_collateral_token_account.as_ref(),
            self.initial_collateral_token_vault.as_ref(),
        ) else {
            return err!(ExchangeError::MissingDepositTokenAccount);
        };
        Ok(CpiContext::new(
            self.token_program.to_account_info(),
            token::Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        ))
    }

    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, system_program::Transfer<'info>> {
        CpiContext::new(
            self.system_program.to_account_info(),
            system_program::Transfer {
                from: self.payer.to_account_info(),
                to: self.order.to_account_info(),
            },
        )
    }

    fn common_tokens(
        &self,
        output_token: &Pubkey,
        include_index_token: bool,
    ) -> Result<BTreeSet<Pubkey>> {
        let mut tokens = BTreeSet::default();
        let ctx = CpiContext::new(
            self.data_store_program.to_account_info(),
            GetMarketMeta {
                market: self.market.to_account_info(),
            },
        );
        let meta = data_store::cpi::get_market_meta(ctx)?.get();
        tokens.insert(meta.long_token_mint);
        tokens.insert(meta.short_token_mint);
        if include_index_token {
            tokens.insert(meta.index_token_mint);
        }
        if let Some(account) = self.initial_collateral_token_account.as_ref() {
            tokens.insert(account.mint);
        }
        if let Some(account) = self.final_output_token_account.as_ref() {
            tokens.insert(account.mint);
        }
        if let Some(account) = self.secondary_output_token_account.as_ref() {
            require!(
                tokens.contains(&account.mint),
                ExchangeError::InvalidSecondaryOutputToken
            );
        }
        require!(
            tokens.contains(output_token),
            ExchangeError::InvalidOutputToken
        );
        Ok(tokens)
    }

    fn handle_tokens_for_increase_order(
        &self,
        output_token: &Pubkey,
        remaining_accounts: &[AccountInfo<'info>],
        length: usize,
    ) -> Result<(BTreeSet<Pubkey>, SwapParams)> {
        let mut tokens = self.common_tokens(output_token, true)?;
        require_gte!(
            remaining_accounts.len(),
            length,
            ExchangeError::NotEnoughRemainingAccounts
        );
        let initial_token = self
            .initial_collateral_token_account
            .as_ref()
            .map(|a| a.mint)
            .ok_or(ExchangeError::MissingDepositTokenAccount)?;
        let swap_path = get_and_validate_swap_path(
            &self.data_store_program,
            &remaining_accounts[..length],
            &initial_token,
            output_token,
            &mut tokens,
        )?;
        Ok((
            tokens,
            SwapParams {
                long_token_swap_path: swap_path,
                short_token_swap_path: vec![],
            },
        ))
    }

    fn handle_tokens_for_decrease_order(
        &self,
        output_token: &Pubkey,
        remaining_accounts: &[AccountInfo<'info>],
        length: usize,
    ) -> Result<(BTreeSet<Pubkey>, SwapParams)> {
        let mut tokens = self.common_tokens(output_token, true)?;
        require_gte!(
            remaining_accounts.len(),
            length,
            ExchangeError::NotEnoughRemainingAccounts
        );
        let final_token = self
            .final_output_token_account
            .as_ref()
            .map(|a| a.mint)
            .ok_or(ExchangeError::MissingDepositTokenAccount)?;
        let swap_path = get_and_validate_swap_path(
            &self.data_store_program,
            &remaining_accounts[..length],
            output_token,
            &final_token,
            &mut tokens,
        )?;
        Ok((
            tokens,
            SwapParams {
                long_token_swap_path: swap_path,
                short_token_swap_path: vec![],
            },
        ))
    }

    fn to_tokens_with_feed(
        &self,
        tokens: impl IntoIterator<Item = Pubkey>,
    ) -> Result<Vec<(Pubkey, Pubkey)>> {
        tokens
            .into_iter()
            .map(|token| {
                let ctx = CpiContext::new(
                    self.data_store_program.to_account_info(),
                    GetTokenConfig {
                        map: self.token_config_map.to_account_info(),
                    },
                );
                let config = data_store::cpi::get_token_config(ctx, self.store.key(), token)?
                    .get()
                    .ok_or(ExchangeError::ResourceNotFound)?;
                Result::Ok((token, config.price_feed))
            })
            .collect::<Result<Vec<_>>>()
    }
}
