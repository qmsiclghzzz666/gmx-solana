use crate::{
    states::{
        revertible::{Revertible, RevertibleSwapMarket},
        status::MarketStatus,
        Factor, HasMarketMeta, ValidateMarketBalances,
    },
    ModelError,
};

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_model::{
    action::Prices, num::Unsigned, BalanceExt, BaseMarketMut, LiquidityMarketExt, PnlFactorKind,
    PoolExt,
};
use gmsol_utils::InitSpace;

use crate::{
    constants,
    states::{
        config::{EntryArgs, MarketConfigBuffer},
        Action, Market, MarketChangeEvent, MarketMeta, Seed, Store, TokenMapAccess, TokenMapHeader,
        TokenMapLoader,
    },
    utils::internal,
    StoreError,
};

/// The accounts definition for [`initialize_market`](crate::gmsol_store::initialize_market).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::initialize_market)*
#[derive(Accounts)]
#[instruction(market_token: Pubkey)]
pub struct InitializeMarket<'info> {
    /// The address authorized to execute this instruction.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// The store account.
    #[account(has_one = token_map)]
    pub store: AccountLoader<'info, Store>,
    /// The market account.
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [
            Market::SEED,
            store.key().as_ref(),
            market_token.as_ref(),
        ],
        bump,
    )]
    pub market: AccountLoader<'info, Market>,
    /// The token map account.
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
    /// The system program.
    pub system_program: Program<'info, System>,
}

/// Initialize the account for [`Market`].
///
/// ## CHECK
/// - Only MARKET_KEEPER can create new market.
pub(crate) fn unchecked_initialize_market(
    ctx: Context<InitializeMarket>,
    market_token_mint: Pubkey,
    index_token_mint: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
    name: &str,
    enable: bool,
) -> Result<()> {
    {
        let token_map = ctx.accounts.token_map.load_token_map()?;
        require!(
            token_map
                .get(&index_token_mint)
                .ok_or(error!(StoreError::RequiredResourceNotFound))?
                .is_enabled(),
            StoreError::InvalidArgument
        );
        require!(
            token_map
                .get(&long_token_mint)
                .ok_or(error!(StoreError::RequiredResourceNotFound))?
                .is_enabled(),
            StoreError::InvalidArgument
        );
        require!(
            token_map
                .get(&short_token_mint)
                .ok_or(error!(StoreError::RequiredResourceNotFound))?
                .is_enabled(),
            StoreError::InvalidArgument
        );
    }
    let market = &ctx.accounts.market;
    market.load_init()?.init(
        ctx.bumps.market,
        ctx.accounts.store.key(),
        name,
        market_token_mint,
        index_token_mint,
        long_token_mint,
        short_token_mint,
        enable,
    )?;
    emit!(MarketChangeEvent {
        address: market.key(),
        action: Action::Init,
    });
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`remove_market`](crate::gmsol_store::remove_market).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::remove_market)*
#[derive(Accounts)]
pub struct RemoveMarket<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [Market::SEED, store.key().as_ref(), market.load()?.meta().market_token_mint.as_ref()],
        bump = market.load()?.bump,
        close = authority,
    )]
    market: AccountLoader<'info, Market>,
}

/// Remove market.
///
/// ## CHECK
/// - Only MARKET_KEEPER can remove market.
pub(crate) fn unchecked_remove_market(ctx: Context<RemoveMarket>) -> Result<()> {
    emit!(MarketChangeEvent {
        address: ctx.accounts.market.key(),
        action: Action::Remove,
    });
    Ok(())
}

impl<'info> internal::Authentication<'info> for RemoveMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`get_validated_market_meta`](crate::gmsol_store::get_validated_market_meta).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::get_validated_market_meta)*
#[derive(Accounts)]
pub struct GetValidatedMarketMeta<'info> {
    pub(crate) store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub(crate) market: AccountLoader<'info, Market>,
}

/// Get the meta of the market after validation.
pub(crate) fn get_validated_market_meta(
    ctx: Context<GetValidatedMarketMeta>,
) -> Result<MarketMeta> {
    let market = ctx.accounts.market.load()?;
    market.validate(&ctx.accounts.store.key())?;
    Ok(*market.meta())
}

#[derive(Accounts)]
pub struct MarketTransferIn<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    pub from_authority: Signer<'info>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut, token::mint = vault.mint, constraint = from.key() != vault.key())]
    pub from: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Transfer some tokens into the market.
///
/// ## CHECK
/// - Only CONTROLLER can transfer in tokens with this method.
pub(crate) fn unchecked_market_transfer_in(
    ctx: Context<MarketTransferIn>,
    amount: u64,
) -> Result<()> {
    use anchor_spl::token;

    ctx.accounts
        .market
        .load()?
        .validate(&ctx.accounts.store.key())?;

    if amount != 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.from_authority.to_account_info(),
                },
            ),
            amount,
        )?;
        let token = &ctx.accounts.vault.mint;
        ctx.accounts
            .market
            .load_mut()?
            .record_transferred_in_by_token(token, amount)?;
    }

    Ok(())
}

impl<'info> internal::Authentication<'info> for MarketTransferIn<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`market_transfer_out`](crate::gmsol_store::market_transfer_out).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::market_transfer_out)*
#[derive(Accounts)]
pub struct MarketTransferOut<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut, token::mint = vault.mint, constraint = to.key() != vault.key())]
    pub to: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Transfer some tokens out of the market.
///
/// ## CHECK
/// - Only CONTROLLER can transfer out from the market.
pub(crate) fn unchecked_market_transfer_out(
    ctx: Context<MarketTransferOut>,
    amount: u64,
) -> Result<()> {
    use crate::utils::internal::TransferUtils;

    ctx.accounts
        .market
        .load()?
        .validate(&ctx.accounts.store.key())?;

    if amount != 0 {
        TransferUtils::new(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.store,
            None,
        )
        .transfer_out(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.to.to_account_info(),
            amount,
        )?;
        let token = &ctx.accounts.vault.mint;
        ctx.accounts
            .market
            .load_mut()?
            .record_transferred_out_by_token(token, amount)?;
    }

    Ok(())
}

impl<'info> internal::Authentication<'info> for MarketTransferOut<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for read-only instructions for market.
#[derive(Accounts)]
pub struct ReadMarket<'info> {
    market: AccountLoader<'info, Market>,
}

/// Get market config by key.
pub(crate) fn get_market_config(ctx: Context<ReadMarket>, key: &str) -> Result<Factor> {
    ctx.accounts.market.load()?.get_config(key).copied()
}

/// Get the meta of the market.
pub(crate) fn get_market_meta(ctx: Context<ReadMarket>) -> Result<MarketMeta> {
    let market = ctx.accounts.market.load()?;
    Ok(*market.meta())
}

/// Get market status.
pub(crate) fn get_market_status(
    ctx: Context<ReadMarket>,
    prices: &Prices<u128>,
    maximize_pnl: bool,
    maximize_pool_value: bool,
) -> Result<MarketStatus> {
    let market = ctx.accounts.market.load()?;
    let status = MarketStatus::from_market(&market, prices, maximize_pnl, maximize_pool_value)
        .map_err(ModelError::from)?;
    Ok(status)
}

/// The accounts definition for read-only instructions for market.
#[derive(Accounts)]
pub struct ReadMarketWithToken<'info> {
    #[account(
        constraint = market.load()?.meta.market_token_mint == market_token.key() @ StoreError::InvalidArgument,
    )]
    market: AccountLoader<'info, Market>,
    market_token: Account<'info, Mint>,
}

/// Get market token price.
pub(crate) fn get_market_token_price(
    ctx: Context<ReadMarketWithToken>,
    prices: &Prices<u128>,
    pnl_factor: PnlFactorKind,
    maximize: bool,
) -> Result<u128> {
    let market = ctx.accounts.market.load()?;
    let liquidity_market = market.as_liquidity_market(&ctx.accounts.market_token);
    let price = liquidity_market
        .market_token_price(prices, pnl_factor, maximize)
        .map_err(ModelError::from)?;
    Ok(price)
}

/// The accounts definition for [`update_market_config`](crate::gmsol_store::update_market_config).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::update_market_config)*
#[derive(Accounts)]
pub struct UpdateMarketConfig<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
}

/// Update market config by key.
///
/// ## CHECK
/// - Only MARKET_KEEPER can udpate the config of market.
pub(crate) fn unchecked_update_market_config(
    ctx: Context<UpdateMarketConfig>,
    key: &str,
    value: Factor,
) -> Result<()> {
    *ctx.accounts.market.load_mut()?.get_config_mut(key)? = value;
    msg!(
        "{}: set {} = {}",
        ctx.accounts.market.load()?.meta.market_token_mint,
        key,
        value
    );
    Ok(())
}

impl<'info> internal::Authentication<'info> for UpdateMarketConfig<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`update_market_config_with_buffer`](crate::gmsol_store::update_market_config_with_buffer).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::update_market_config_with_buffer)*
#[derive(Accounts)]
pub struct UpdateMarketConfigWithBuffer<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
    #[account(mut, has_one = store, has_one = authority @ StoreError::PermissionDenied)]
    buffer: Account<'info, MarketConfigBuffer>,
}

/// Update market config with buffer.
///
/// ## CHECK
/// - Only MARKET_KEEPER can udpate the config of market.
pub(crate) fn unchecked_update_market_config_with_buffer(
    ctx: Context<UpdateMarketConfigWithBuffer>,
) -> Result<()> {
    let buffer = &ctx.accounts.buffer;
    require_gt!(
        buffer.expiry,
        Clock::get()?.unix_timestamp,
        StoreError::InvalidArgument
    );
    ctx.accounts
        .market
        .load_mut()?
        .update_config_with_buffer(buffer)?;
    msg!(
        "{} updated with buffer {}",
        ctx.accounts.market.load()?.description()?,
        buffer.key()
    );
    Ok(())
}

impl<'info> internal::Authentication<'info> for UpdateMarketConfigWithBuffer<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`initialize_market_config_buffer`](crate::gmsol_store::initialize_market_config_buffer).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::initialize_market_config_buffer)*
#[derive(Accounts)]
pub struct InitializeMarketConfigBuffer<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(init, payer = authority, space = 8 + MarketConfigBuffer::init_space(0))]
    buffer: Account<'info, MarketConfigBuffer>,
    system_program: Program<'info, System>,
}

/// Initialize a market config buffer account.
pub(crate) fn initialize_market_config_buffer(
    ctx: Context<InitializeMarketConfigBuffer>,
    expire_after_secs: u32,
) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    buffer.authority = ctx.accounts.authority.key();
    buffer.store = ctx.accounts.store.key();
    buffer.expiry = Clock::get()?
        .unix_timestamp
        .saturating_add_unsigned(expire_after_secs as u64);
    Ok(())
}

/// The accounts definition for [`set_market_config_buffer_authority`](crate::gmsol_store::set_market_config_buffer_authority).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::set_market_config_buffer_authority)*
#[derive(Accounts)]
pub struct SetMarketConfigBufferAuthority<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut, has_one = authority @ StoreError::PermissionDenied)]
    buffer: Account<'info, MarketConfigBuffer>,
}

/// Set the authority of the buffer account.
pub(crate) fn set_market_config_buffer_authority(
    ctx: Context<SetMarketConfigBufferAuthority>,
    new_authority: Pubkey,
) -> Result<()> {
    ctx.accounts.buffer.authority = new_authority;
    Ok(())
}

/// The accounts definition for [`close_market_config_buffer`](crate::gmsol_store::close_market_config_buffer).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::close_market_config_buffer)*
#[derive(Accounts)]
pub struct CloseMarketConfigBuffer<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(mut, close = receiver, has_one = authority @ StoreError::PermissionDenied)]
    buffer: Account<'info, MarketConfigBuffer>,
    /// CHECK: Only used to receive funds after closing the buffer account.
    #[account(mut)]
    receiver: UncheckedAccount<'info>,
}

/// Close the buffer account.
pub(crate) fn close_market_config_buffer(_ctx: Context<CloseMarketConfigBuffer>) -> Result<()> {
    Ok(())
}

/// The accounts definition for [`push_to_market_config_buffer`](crate::gmsol_store::push_to_market_config_buffer).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::push_to_market_config_buffer)*
#[derive(Accounts)]
#[instruction(new_configs: Vec<(String, Factor)>)]
pub struct PushToMarketConfigBuffer<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority @ StoreError::PermissionDenied,
        realloc = 8 + buffer.space_after_push(new_configs.len()),
        realloc::payer = authority,
        realloc::zero = false,
    )]
    buffer: Account<'info, MarketConfigBuffer>,
    system_program: Program<'info, System>,
}

/// Push to the buffer account.
pub(crate) fn push_to_market_config_buffer(
    ctx: Context<PushToMarketConfigBuffer>,
    new_configs: Vec<EntryArgs>,
) -> Result<()> {
    let buffer = &mut ctx.accounts.buffer;
    for entry in new_configs {
        buffer.push(entry.try_into()?);
    }
    Ok(())
}

/// The accounts definition for [`toggle_market`](crate::gmsol_store::toggle_market).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::toggle_market)*
#[derive(Accounts)]
pub struct ToggleMarket<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
}

/// Toggle Market.
///
/// ## CHECK
/// - Only MARKET_KEEPER can toggle market.
pub(crate) fn unchecked_toggle_market(ctx: Context<ToggleMarket>, enable: bool) -> Result<()> {
    ctx.accounts.market.load_mut()?.set_enabled(enable);
    Ok(())
}

impl<'info> internal::Authentication<'info> for ToggleMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`toggle_gt_minting`](crate::gmsol_store::toggle_gt_minting).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::toggle_gt_minting)*
#[derive(Accounts)]
pub struct ToggleGTMinting<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
}

/// Toggle GT Minting.
///
/// ## CHECK
/// - Only MARKET_KEEPER can use this instruction.
pub(crate) fn unchecked_toggle_gt_minting(
    ctx: Context<ToggleGTMinting>,
    enable: bool,
) -> Result<()> {
    ctx.accounts
        .market
        .load_mut()?
        .set_is_gt_minting_enbaled(enable);
    Ok(())
}

impl<'info> internal::Authentication<'info> for ToggleGTMinting<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

/// The accounts definition for [`claim_fees_from_market`](crate::gmsol_store::claim_fees_from_market).
///
/// *[See also the documentation for the instruction.](crate::gmsol_store::claim_fees_from_market)*
#[derive(Accounts)]
pub struct ClaimFeesFromMarket<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
}

/// Claim fees from the market.
///
/// ## CHECK
/// - Only CONTROLLER can claim fees.
pub(crate) fn unchecked_claim_fees_from_market(
    ctx: Context<ClaimFeesFromMarket>,
    token: &Pubkey,
) -> gmsol_model::Result<u64> {
    let mut market = RevertibleSwapMarket::from_market((&ctx.accounts.market).try_into()?)?;
    let is_long_token = market.market_meta().to_token_side(token)?;
    let pool = market.claimable_fee_pool_mut()?;

    // Saturating claim all fees from the pool.
    let amount: u64 = pool
        .amount(is_long_token)?
        .min(u128::from(u64::MAX))
        .try_into()
        .expect("must success");

    let delta = (u128::from(amount)).to_opposite_signed()?;
    pool.apply_delta_amount(is_long_token, &delta)?;

    market.validate_market_balance_for_the_given_token(token, amount)?;
    market.commit();

    msg!(
        "Claimed `{}` {} from the {} market",
        amount,
        token,
        ctx.accounts.market.load()?.meta.market_token_mint
    );
    Ok(amount)
}

impl<'info> internal::Authentication<'info> for ClaimFeesFromMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
