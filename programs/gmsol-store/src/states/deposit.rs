use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::StoreError;

use super::{
    common::{swap::SwapParamsV2, token::TokenAndAccount, SwapParams, TokenRecord, TokensWithFeed},
    Market, NonceBytes, Seed,
};

/// Deposit.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Deposit {
    /// Fixed part.
    pub fixed: Fixed,
    /// Dynamic part.
    pub dynamic: Dynamic,
}

impl Deposit {
    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord], swap_params: &SwapParams) -> usize {
        Fixed::INIT_SPACE + Dynamic::init_space(tokens_with_feed, swap_params)
    }

    /// Get min market tokens.
    pub fn min_market_tokens(&self) -> u64 {
        self.fixed.tokens.params.min_market_tokens
    }

    pub(crate) fn validate_min_market_tokens(&self, minted: u64) -> Result<()> {
        require_gte!(
            minted,
            self.min_market_tokens(),
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }
}

/// Fixed part of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Fixed {
    /// Store.
    pub store: Pubkey,
    /// Market.
    pub market: Pubkey,
    /// Action id.
    pub id: u64,
    /// The slot that the deposit was last updated at.
    pub updated_at_slot: u64,
    /// The time that the deposit was last updated at.
    pub updated_at: i64,
    /// The bump seed.
    pub bump: u8,
    /// The nonce bytes for this deposit.
    pub nonce: [u8; 32],
    /// Senders.
    pub senders: Senders,
    /// The receivers of the deposit.
    pub receivers: Receivers,
    /// Tokens config.
    pub tokens: Tokens,
    reserved: [u8; 128],
}

/// Senders of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Senders {
    /// The user depositing liquidity.
    pub user: Pubkey,
    /// Initial long token account.
    pub initial_long_token_account: Option<Pubkey>,
    /// Initial short token account.
    pub initial_short_token_account: Option<Pubkey>,
}

/// Tokens config of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tokens {
    /// The market token of the market.
    pub market_token: Pubkey,
    /// Initial long token.
    pub initial_long_token: Option<Pubkey>,
    /// Initial short token.
    pub initial_short_token: Option<Pubkey>,
    /// Params.
    pub params: TokenParams,
}

/// Dynamic part of [`Deposit`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Dynamic {
    /// Tokens with feed.
    pub tokens_with_feed: TokensWithFeed,
    /// Swap params.
    pub swap_params: SwapParams,
}

impl Dynamic {
    fn init_space(tokens_with_feed: &[TokenRecord], swap_params: &SwapParams) -> usize {
        TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap_params.long_token_swap_path.len(),
                swap_params.short_token_swap_path.len(),
            )
    }
}

impl Seed for Deposit {
    const SEED: &'static [u8] = b"deposit";
}

impl Deposit {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn init(
        &mut self,
        bump: u8,
        id: u64,
        store: Pubkey,
        market: &AccountLoader<Market>,
        nonce: NonceBytes,
        tokens_with_feed: Vec<TokenRecord>,
        user: Pubkey,
        initial_long_token_account: Option<&Account<TokenAccount>>,
        initial_short_token_account: Option<&Account<TokenAccount>>,
        receivers: Receivers,
        token_params: TokenParams,
        swap_params: SwapParams,
    ) -> Result<()> {
        let clock = Clock::get()?;
        *self = Self {
            fixed: Fixed {
                id,
                bump,
                store,
                nonce,
                updated_at_slot: clock.slot,
                updated_at: clock.unix_timestamp,
                market: market.key(),
                senders: Senders {
                    user,
                    initial_long_token_account: initial_long_token_account
                        .as_ref()
                        .map(|a| a.key()),
                    initial_short_token_account: initial_short_token_account
                        .as_ref()
                        .map(|a| a.key()),
                },
                receivers,
                tokens: Tokens {
                    market_token: market.load()?.meta().market_token_mint,
                    initial_long_token: initial_long_token_account.as_ref().map(|a| a.mint),
                    initial_short_token: initial_short_token_account.as_ref().map(|a| a.mint),
                    params: token_params,
                },
                reserved: [0; 128],
            },
            dynamic: Dynamic {
                tokens_with_feed: TokensWithFeed::try_from_vec(tokens_with_feed)?,
                swap_params,
            },
        };
        Ok(())
    }
}

/// The receivers of the deposit.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Receivers {
    /// The address to send the liquidity tokens to.
    pub receiver: Pubkey,
    /// The ui fee receiver.
    pub ui_fee_receiver: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenParams {
    /// The amount of long tokens to deposit.
    pub initial_long_token_amount: u64,
    /// The amount of short tokens to deposit.
    pub initial_short_token_amount: u64,
    /// The minimum acceptable number of liquidity tokens.
    pub min_market_tokens: u64,
    /// Whether to unwrap the native token.
    pub should_unwrap_native_token: bool,
}

/// Deposit V2.
#[account(zero_copy)]
pub struct DepositV2 {
    /// Action id.
    pub(crate) id: u64,
    /// Store.
    pub(crate) store: Pubkey,
    /// Market.
    pub(crate) market: Pubkey,
    /// Owner.
    pub(crate) owner: Pubkey,
    /// Nonce bytes.
    pub(crate) nonce: [u8; 32],
    /// The bump seed.
    pub(crate) bump: u8,
    padding_0: [u8; 7],
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Deposit params.
    pub(crate) params: DepositParams,
    /// Swap params.
    pub(crate) swap: SwapParamsV2,
    padding_1: [u8; 4],
    reserve: [u8; 128],
}

/// Token Accounts.
#[account(zero_copy)]
pub struct TokenAccounts {
    /// Initial long token accounts.
    pub(crate) initial_long_token: TokenAndAccount,
    /// Initial short token accounts.
    pub(crate) initial_short_token: TokenAndAccount,
    /// Market token account.
    pub(crate) market_token: TokenAndAccount,
}

/// Deposit Params.
#[account(zero_copy)]
pub struct DepositParams {
    /// The amount of initial long tokens to deposit.
    pub(crate) initial_long_token_amount: u64,
    /// The amount of initial short tokens to deposit.
    pub(crate) initial_short_token_amount: u64,
    /// The minimum acceptable amount of market tokens to receive.
    pub(crate) min_market_token_amount: u64,
    /// Max execution fee.
    pub(crate) max_execution_lamports: u64,
    reserved: [u8; 64],
}

impl Default for DepositParams {
    fn default() -> Self {
        Self {
            initial_long_token_amount: 0,
            initial_short_token_amount: 0,
            min_market_token_amount: 0,
            max_execution_lamports: DepositV2::MIN_EXECUTION_LAMPORTS,
            reserved: [0; 64],
        }
    }
}

impl DepositV2 {
    /// Seed.
    pub const SEED: &'static [u8] = b"deposit";

    /// Max execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    /// Init Space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();
}
