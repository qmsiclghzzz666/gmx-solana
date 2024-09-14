use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::StoreError;

use super::{
    common::{
        action::{ActionHeader, ActionSigner},
        swap::SwapParamsV2,
        token::TokenAndAccount,
        SwapParams, TokenRecord, TokensWithFeed,
    },
    Market, NonceBytes, Seed,
};

/// Withdrawal.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Withdrawal {
    /// Fixed part.
    pub fixed: Box<Fixed>,
    /// Dynamic part.
    pub dynamic: Dynamic,
}

impl Withdrawal {
    pub(crate) fn init_space(tokens_with_feed: &[TokenRecord], swap: &SwapParams) -> usize {
        Fixed::INIT_SPACE + Dynamic::init_space(tokens_with_feed, swap)
    }

    pub(crate) fn validate_output_amounts(
        &self,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let params = &self.fixed.tokens.params;
        require_gte!(
            long_amount,
            params.min_long_token_amount,
            StoreError::InsufficientOutputAmount
        );
        require_gte!(
            short_amount,
            params.min_short_token_amount,
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }
}

/// Fixed part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Fixed {
    /// Store.
    pub store: Pubkey,
    /// The market on which the withdrawal will be executed.
    pub market: Pubkey,
    /// Action id.
    pub id: u64,
    /// The slot that the withdrawal was last updated at.
    pub updated_at_slot: u64,
    /// The time that the withdrawal was last updated at.
    pub updated_at: i64,
    /// The bump seed.
    pub bump: u8,
    /// The nonce bytes for this withdrawal.
    pub nonce: [u8; 32],
    /// The user to withdraw for.
    pub user: Pubkey,
    /// The market token account.
    pub market_token_account: Pubkey,
    /// Receivers.
    pub receivers: Receivers,
    /// Tokens config.
    pub tokens: Tokens,
    reserved: [u8; 128],
}

/// Dynamic part of [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Dynamic {
    /// Tokens with feed.
    pub tokens_with_feed: TokensWithFeed,
    /// Swap params.
    pub swap: SwapParams,
}

impl Dynamic {
    fn init_space(tokens_with_feed: &[TokenRecord], swap: &SwapParams) -> usize {
        TokensWithFeed::init_space(tokens_with_feed)
            + SwapParams::init_space(
                swap.long_token_swap_path.len(),
                swap.short_token_swap_path.len(),
            )
    }
}

/// Fees and tokens receivers for [`Withdrawal`]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Receivers {
    /// The ui fee receiver.
    pub ui_fee_receiver: Pubkey,
    /// Token account for receiving the final long tokens.
    pub final_long_token_receiver: Pubkey,
    /// Token account for receiving the final short tokens.
    pub final_short_token_receiver: Pubkey,
}

/// Tokens config for [`Withdrawal`].
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tokens {
    /// Params.
    pub params: TokenParams,
    /// The market token to burn.
    pub market_token: Pubkey,
    /// The final long token to receive.
    pub final_long_token: Pubkey,
    /// The final short token to receive.
    pub final_short_token: Pubkey,
    /// The amount of market tokens that will be withdrawn.
    pub market_token_amount: u64,
}

/// Tokens params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenParams {
    /// The minimum amount of final long tokens that must be withdrawn.
    pub min_long_token_amount: u64,
    /// The minimum amount of final short tokens that must be withdrawn.
    pub min_short_token_amount: u64,
    /// Whether to unwrap the native token.
    pub should_unwrap_native_token: bool,
}

impl Seed for Withdrawal {
    const SEED: &'static [u8] = b"withdrawal";
}

impl Withdrawal {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn init(
        &mut self,
        bump: u8,
        id: u64,
        store: Pubkey,
        nonce: NonceBytes,
        user: Pubkey,
        market: &AccountLoader<Market>,
        market_token_account: Pubkey,
        market_token_amount: u64,
        token_params: TokenParams,
        swap_params: SwapParams,
        tokens_with_feed: Vec<TokenRecord>,
        final_long_token_receiver: &Account<TokenAccount>,
        final_short_token_receiver: &Account<TokenAccount>,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        *self = Self {
            fixed: Box::new(Fixed {
                id,
                bump,
                store,
                nonce,
                updated_at_slot: clock.slot,
                updated_at: clock.unix_timestamp,
                user,
                market_token_account,
                market: market.key(),
                receivers: Receivers {
                    ui_fee_receiver,
                    final_long_token_receiver: final_long_token_receiver.key(),
                    final_short_token_receiver: final_short_token_receiver.key(),
                },
                tokens: Tokens {
                    params: token_params,
                    market_token: market.load()?.meta().market_token_mint,
                    final_long_token: final_long_token_receiver.mint,
                    final_short_token: final_short_token_receiver.mint,
                    market_token_amount,
                },
                reserved: [0; 128],
            }),
            dynamic: Dynamic {
                tokens_with_feed: TokensWithFeed::try_from_records(tokens_with_feed)?,
                swap: swap_params,
            },
        };
        Ok(())
    }
}

/// Withdrawal.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WithdrawalV2 {
    /// Action header.
    pub(crate) header: ActionHeader,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Withdrawal params.
    pub(crate) params: WithdrawalParams,
    /// Swap params.
    pub(crate) swap: SwapParamsV2,
    padding_1: [u8; 4],
    pub(crate) updated_at: i64,
    pub(crate) updated_at_slot: u64,
    reserve: [u8; 128],
}

impl WithdrawalV2 {
    /// Seed.
    pub const SEED: &'static [u8] = b"withdrawal";

    /// Init space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Max execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    /// Get the action header.
    pub fn header(&self) -> &ActionHeader {
        &self.header
    }

    /// Get tokens and accounts.
    pub fn tokens(&self) -> &TokenAccounts {
        &self.tokens
    }

    /// Get action signer.
    pub fn signer(&self) -> ActionSigner {
        self.header.signer(Self::SEED)
    }

    /// Get the swap params.
    pub fn swap(&self) -> &SwapParamsV2 {
        &self.swap
    }

    pub(crate) fn validate_output_amounts(
        &self,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let params = &self.params;
        require_gte!(
            long_amount,
            params.min_long_token_amount,
            StoreError::InsufficientOutputAmount
        );
        require_gte!(
            short_amount,
            params.min_short_token_amount,
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }
}

/// Token Accounts.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct TokenAccounts {
    /// Final long token accounts.
    pub(crate) final_long_token: TokenAndAccount,
    /// Final short token accounts.
    pub(crate) final_short_token: TokenAndAccount,
    /// Market token account.
    pub(crate) market_token: TokenAndAccount,
}

impl TokenAccounts {
    /// Get market token.
    pub fn market_token(&self) -> Pubkey {
        self.market_token.token().expect("must exist")
    }

    /// Get market token account.
    pub fn market_token_account(&self) -> Pubkey {
        self.market_token.account().expect("must exist")
    }

    /// Get final_long token.
    pub fn final_long_token(&self) -> Pubkey {
        self.final_long_token.token().expect("must exist")
    }

    /// Get final_long token account.
    pub fn final_long_token_account(&self) -> Pubkey {
        self.final_long_token.account().expect("must exist")
    }

    /// Get final_short token.
    pub fn final_short_token(&self) -> Pubkey {
        self.final_short_token.token().expect("must exist")
    }

    /// Get final_short token account.
    pub fn final_short_token_account(&self) -> Pubkey {
        self.final_short_token.account().expect("must exist")
    }
}

/// Withdrawal params.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct WithdrawalParams {
    /// Market token amount to burn.
    pub market_token_amount: u64,
    /// The minimum acceptable amount of final long tokens to receive.
    pub min_long_token_amount: u64,
    /// The minimum acceptable amount of final short tokens to receive.
    pub min_short_token_amount: u64,
    /// Max execution fee.
    pub max_execution_lamports: u64,
    reserved: [u8; 64],
}

impl Default for WithdrawalParams {
    fn default() -> Self {
        Self {
            max_execution_lamports: WithdrawalV2::MIN_EXECUTION_LAMPORTS,
            reserved: [0; 64],
            market_token_amount: 0,
            min_long_token_amount: 0,
            min_short_token_amount: 0,
        }
    }
}
