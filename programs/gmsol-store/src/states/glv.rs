use std::collections::HashSet;

use anchor_lang::prelude::*;

use crate::CoreError;

use super::{
    common::{
        action::{Action, ActionHeader},
        swap::{unpack_markets, SwapParams},
        token::TokenAndAccount,
    },
    Seed,
};

/// Glv.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Glv {
    version: u8,
    /// Bump seed.
    pub(crate) bump: u8,
    /// Index.
    pub(crate) index: u8,
    /// Num of markets.
    pub(crate) num_markets: u8,
    padding: [u8; 4],
    pub(crate) store: Pubkey,
    pub(crate) glv_token: Pubkey,
    pub(crate) long_token: Pubkey,
    pub(crate) short_token: Pubkey,
    reserve: [u8; 256],
    market_tokens: [Pubkey; Glv::MAX_ALLOWED_NUMBER_OF_MARKETS],
}

impl Seed for Glv {
    const SEED: &'static [u8] = b"glv";
}

impl Glv {
    /// Init space.
    pub const INIT_SPACE: usize = std::mem::size_of::<Self>();

    /// GLV token seed.
    pub const GLV_TOKEN_SEED: &'static [u8] = b"glv_token";

    /// Max allowed number of markets.
    pub const MAX_ALLOWED_NUMBER_OF_MARKETS: usize = 128;

    /// Find GLV token address.
    pub fn find_glv_token_pda(store: &Pubkey, index: u8, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::GLV_TOKEN_SEED, store.as_ref(), &[index]],
            program_id,
        )
    }

    /// Find GLV address.
    pub fn find_glv_pda(glv_token: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED, glv_token.as_ref()], program_id)
    }

    /// Initialize the [`Glv`] account.
    ///
    /// # CHECK
    /// - The [`Glv`] account must be uninitialized.
    /// - The `bump` must be the bump deriving the address of the [`Glv`] account.
    /// - The `glv_token` must be used to derive the address of the [`Glv`] account.
    /// - The market tokens must be valid and unique, and their corresponding markets
    ///   must use the given tokens as long token and short token.
    /// - The `store` must be the address of the store owning the corresponding markets.
    ///
    /// # Errors
    /// - The `glv_token` address must be derived from [`GLV_TOKEN_SEED`](Self::GLV_TOKEN_SEED), `store` and `index`.
    /// - The total number of the market tokens must not exceed the max allowed number of markets.
    pub(crate) fn unchecked_init(
        &mut self,
        bump: u8,
        index: u8,
        store: &Pubkey,
        glv_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
        market_tokens: &HashSet<Pubkey>,
    ) -> Result<()> {
        let expected_glv_token = Self::find_glv_token_pda(store, index, &crate::ID).0;
        require_eq!(expected_glv_token, *glv_token, CoreError::InvalidArgument);

        self.version = 0;
        self.bump = bump;
        self.index = index;
        self.store = *store;
        self.glv_token = *glv_token;
        self.long_token = *long_token;
        self.short_token = *short_token;

        for (idx, market_token) in market_tokens.iter().enumerate() {
            self.num_markets += 1;
            require_gte!(
                Self::MAX_ALLOWED_NUMBER_OF_MARKETS,
                self.num_markets as usize,
                CoreError::ExceedMaxLengthLimit
            );
            self.market_tokens[idx] = *market_token;
        }
        Ok(())
    }

    pub(crate) fn process_and_validate_markets_for_init<'info>(
        markets: &'info [AccountInfo<'info>],
        store: &Pubkey,
    ) -> Result<(Pubkey, Pubkey, HashSet<Pubkey>)> {
        let mut tokens = None;

        let mut market_tokens = HashSet::default();
        for market in unpack_markets(markets) {
            let market = market?;
            let market = market.load()?;
            market.validate(store)?;
            let meta = market.meta();
            match &mut tokens {
                Some((long_token, short_token)) => {
                    require_eq!(
                        *long_token,
                        meta.long_token_mint,
                        CoreError::TokenMintMismatched
                    );
                    require_eq!(
                        *short_token,
                        meta.short_token_mint,
                        CoreError::TokenMintMismatched
                    );
                }
                none => {
                    *none = Some((meta.long_token_mint, meta.short_token_mint));
                }
            }
            require!(
                market_tokens.insert(meta.market_token_mint),
                CoreError::InvalidArgument
            );
        }

        if let Some((long_token, short_token)) = tokens {
            Ok((long_token, short_token, market_tokens))
        } else {
            err!(CoreError::InvalidArgument)
        }
    }

    /// Get the version of the [`Glv`] account format.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get the index of the glv token.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Get the store address.
    pub fn store(&self) -> &Pubkey {
        &self.store
    }

    /// Get the GLV token address.
    pub fn glv_token(&self) -> &Pubkey {
        &self.glv_token
    }

    /// Get the long token address.
    pub fn long_token(&self) -> &Pubkey {
        &self.long_token
    }

    /// Get the short token address.
    pub fn short_token(&self) -> &Pubkey {
        &self.short_token
    }

    /// Get all market tokens.
    pub fn market_tokens(&self) -> &[Pubkey] {
        &self.market_tokens[0..(self.num_markets as usize)]
    }
}

/// Glv Deposit.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct GlvDeposit {
    /// Header.
    pub(crate) header: ActionHeader,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Params.
    pub(crate) params: GlvDepositParams,
    /// Swap params.
    pub(crate) swap: SwapParams,
    padding_1: [u8; 4],
    reserve: [u8; 128],
}

impl Action for GlvDeposit {
    const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    fn header(&self) -> &ActionHeader {
        &self.header
    }
}

impl Seed for GlvDeposit {
    const SEED: &'static [u8] = b"glv_deposit";
}

impl gmsol_utils::InitSpace for GlvDeposit {
    const INIT_SPACE: usize = core::mem::size_of::<Self>();
}

/// Token Accounts.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct TokenAccounts {
    /// Initial long token and account.
    pub initial_long_token: TokenAndAccount,
    /// Initial short token and account.
    pub initial_short_token: TokenAndAccount,
    /// Market token and account.
    pub(crate) market_token: TokenAndAccount,
    /// GLV token and account.
    pub(crate) glv_token: TokenAndAccount,
}

impl TokenAccounts {
    /// Get market token.
    pub fn market_token(&self) -> Pubkey {
        self.market_token
            .token()
            .expect("uninitialized GLV Deposit account")
    }

    /// Get market token account.
    pub fn market_token_account(&self) -> Pubkey {
        self.market_token
            .account()
            .expect("uninitalized GLV Deposit account")
    }

    /// Get GLV token.
    pub fn glv_token(&self) -> Pubkey {
        self.glv_token
            .token()
            .expect("uninitialized GLV Deposit account")
    }

    /// Get GLV token account.
    pub fn glv_token_account(&self) -> Pubkey {
        self.glv_token
            .account()
            .expect("uninitalized GLV Deposit account")
    }
}

/// GLV Deposit Params.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct GlvDepositParams {
    /// The amount of initial long tokens to deposit.
    pub(crate) initial_long_token_amount: u64,
    /// The amount of initial short tokens to deposit.
    pub(crate) initial_short_token_amount: u64,
    /// The amount of market tokens to deposit.
    pub(crate) market_token_amount: u64,
    /// The minimum acceptable amount of glv tokens to receive.
    pub(crate) min_glv_token_amount: u64,
    reserved: [u8; 64],
}
