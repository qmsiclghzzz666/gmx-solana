use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use anchor_spl::associated_token::get_associated_token_address;
use data_store::states::{Chainlink, Deposit, Market, NonceBytes, Seed};
use exchange::{accounts, instruction, instructions::CreateDepositParams, utils::ControllerSeeds};

use crate::store::{
    data_store::{find_market_address, find_market_vault_address, find_token_config_map},
    roles::find_roles_address,
};

use super::generate_nonce;

/// Create PDA for deposit.
pub fn find_deposit_address(store: &Pubkey, user: &Pubkey, nonce: &NonceBytes) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Deposit::SEED, store.as_ref(), user.as_ref(), nonce],
        &data_store::id(),
    )
}

/// Create Deposit Builder.
pub struct CreateDepositBuilder<'a, C> {
    program: &'a Program<C>,
    store: Pubkey,
    market_token: Pubkey,
    receiver: Option<Pubkey>,
    ui_fee_receiver: Pubkey,
    execution_fee: u64,
    long_token_swap_path: Vec<Pubkey>,
    short_token_swap_path: Vec<Pubkey>,
    initial_long_token: Option<Pubkey>,
    initial_short_token: Option<Pubkey>,
    initial_long_token_account: Option<Pubkey>,
    initial_short_token_account: Option<Pubkey>,
    initial_long_token_amount: u64,
    initial_short_token_amount: u64,
    min_market_token: u64,
    should_unwrap_native_token: bool,
    nonce: Option<NonceBytes>,
}

impl<'a, C, S> CreateDepositBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(program: &'a Program<C>, store: Pubkey, market_token: Pubkey) -> Self {
        Self {
            program,
            store,
            nonce: None,
            market_token,
            receiver: None,
            ui_fee_receiver: Pubkey::default(),
            execution_fee: 0,
            long_token_swap_path: vec![],
            short_token_swap_path: vec![],
            initial_long_token: None,
            initial_short_token: None,
            initial_long_token_account: None,
            initial_short_token_account: None,
            initial_long_token_amount: 0,
            initial_short_token_amount: 0,
            min_market_token: 0,
            should_unwrap_native_token: false,
        }
    }

    /// Set the token account for receiving minted market tokens.
    ///
    /// Defaults to use associated token account.
    pub fn receiver(&mut self, token_account: &Pubkey) -> &mut Self {
        self.receiver = Some(*token_account);
        self
    }

    /// Set min market token to mint.
    pub fn min_market_token(&mut self, amount: u64) -> &mut Self {
        self.min_market_token = amount;
        self
    }

    /// Set extra exectuion fee allowed to use.
    ///
    ///  /// Defaults to `0` means only allowed to use at most `rent-exempt` amount of fee.
    pub fn execution_fee(&mut self, fee: u64) -> &mut Self {
        self.execution_fee = fee;
        self
    }

    /// Set long swap path.
    pub fn long_token_swap_path(&mut self, market_tokens: Vec<Pubkey>) -> &mut Self {
        self.long_token_swap_path = market_tokens;
        self
    }

    /// Set short swap path.
    pub fn short_token_swap_path(&mut self, market_tokens: Vec<Pubkey>) -> &mut Self {
        self.short_token_swap_path = market_tokens;
        self
    }

    fn get_or_find_associated_initial_long_token_account(
        &self,
        token: Option<&Pubkey>,
    ) -> Option<Pubkey> {
        let token = token?;
        match self.initial_long_token_account {
            Some(account) => Some(account),
            None => Some(get_associated_token_address(&self.program.payer(), token)),
        }
    }

    fn get_or_find_associated_initial_short_token_account(
        &self,
        token: Option<&Pubkey>,
    ) -> Option<Pubkey> {
        let token = token?;
        match self.initial_short_token_account {
            Some(account) => Some(account),
            None => Some(get_associated_token_address(&self.program.payer(), token)),
        }
    }

    async fn get_or_fetch_initial_tokens(
        &self,
        market: &Pubkey,
    ) -> crate::Result<(Option<Pubkey>, Option<Pubkey>)> {
        let res = match (
            self.initial_long_token,
            self.initial_long_token_amount,
            self.initial_short_token,
            self.initial_short_token_amount,
        ) {
            (Some(long_token), _, Some(short_token), _) => (Some(long_token), Some(short_token)),
            (_, 0, _, 0) => {
                return Err(crate::Error::EmptyDeposit);
            }
            (None, 0, Some(short_token), _) => (None, Some(short_token)),
            (Some(long_token), _, None, 0) => (Some(long_token), None),
            (mut long_token, long_amount, mut short_token, short_amount) => {
                debug_assert!(
                    (long_token.is_none() && long_amount != 0)
                        || (short_token.is_none() && short_amount != 0)
                );
                let market: Market = self.program.account(*market).await?;
                if long_amount != 0 {
                    long_token = Some(market.meta().long_token_mint);
                }
                if short_amount != 0 {
                    short_token = Some(market.meta().short_token_mint);
                }
                (long_token, short_token)
            }
        };
        Ok(res)
    }

    /// Set the initial long token params for deposit.
    pub fn long_token(
        &mut self,
        amount: u64,
        token: Option<&Pubkey>,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.initial_long_token = token.copied();
        self.initial_long_token_amount = amount;
        self.initial_long_token_account = token_account.copied();
        self
    }

    /// Set the initial short token params for deposit.
    pub fn short_token(
        &mut self,
        amount: u64,
        token: Option<&Pubkey>,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.initial_short_token = token.cloned();
        self.initial_short_token_amount = amount;
        self.initial_short_token_account = token_account.copied();
        self
    }

    fn get_receiver(&self) -> Pubkey {
        match self.receiver {
            Some(token_account) => token_account,
            None => anchor_spl::associated_token::get_associated_token_address(
                &self.program.payer(),
                &self.market_token,
            ),
        }
    }

    /// Build a [`RequestBuilder`] and return deposit address.
    pub async fn build_with_address(&self) -> crate::Result<(RequestBuilder<'a, C>, Pubkey)> {
        let receiver = self.get_receiver();
        let Self {
            program,
            store,
            nonce,
            market_token,
            ui_fee_receiver,
            execution_fee,
            long_token_swap_path,
            short_token_swap_path,
            initial_long_token_amount,
            initial_short_token_amount,
            min_market_token,
            should_unwrap_native_token,
            ..
        } = self;
        let nonce = nonce.unwrap_or_else(generate_nonce);
        let payer = program.payer();
        let deposit = find_deposit_address(store, &payer, &nonce).0;
        let (_, authority) = ControllerSeeds::find_with_address(store);
        let only_controller = find_roles_address(store, &authority).0;
        let market = find_market_address(store, market_token).0;
        let (long_token, short_token) = self.get_or_fetch_initial_tokens(&market).await?;
        let initial_long_token_account =
            self.get_or_find_associated_initial_long_token_account(long_token.as_ref());
        let initial_short_token_account =
            self.get_or_find_associated_initial_short_token_account(short_token.as_ref());
        let builder = program
            .request()
            .accounts(accounts::CreateDeposit {
                authority,
                store: *store,
                only_controller,
                data_store_program: data_store::id(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
                deposit,
                payer,
                receiver,
                token_config_map: find_token_config_map(store).0,
                market,
                initial_long_token_account,
                initial_short_token_account,
                long_token_deposit_vault: long_token
                    .map(|token| find_market_vault_address(store, &token).0),
                short_token_deposit_vault: short_token
                    .map(|token| find_market_vault_address(store, &token).0),
            })
            .args(instruction::CreateDeposit {
                nonce,
                params: CreateDepositParams {
                    ui_fee_receiver: *ui_fee_receiver,
                    execution_fee: *execution_fee,
                    long_token_swap_length: long_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::NumberOutOfRange)?,
                    short_token_swap_length: short_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::NumberOutOfRange)?,
                    initial_long_token_amount: *initial_long_token_amount,
                    initial_short_token_amount: *initial_short_token_amount,
                    min_market_token: *min_market_token,
                    should_unwrap_native_token: *should_unwrap_native_token,
                },
            })
            .accounts(
                long_token_swap_path
                    .iter()
                    .chain(short_token_swap_path.iter())
                    .map(|mint| AccountMeta {
                        pubkey: find_market_address(store, mint).0,
                        is_signer: false,
                        is_writable: false,
                    })
                    .collect::<Vec<_>>(),
            );
        Ok((builder, deposit))
    }
}

#[derive(Clone, Copy)]
struct CancelDepositHint {
    initial_long_token: Option<Pubkey>,
    initial_short_token: Option<Pubkey>,
    initial_long_token_account: Option<Pubkey>,
    initial_short_token_account: Option<Pubkey>,
}

impl<'a> From<&'a Deposit> for CancelDepositHint {
    fn from(deposit: &'a Deposit) -> Self {
        Self {
            initial_long_token: deposit.fixed.tokens.initial_long_token,
            initial_short_token: deposit.fixed.tokens.initial_short_token,
            initial_long_token_account: deposit.fixed.senders.initial_long_token_account,
            initial_short_token_account: deposit.fixed.senders.initial_short_token_account,
        }
    }
}

/// Cancel Deposit Builder.
pub struct CancelDepositBuilder<'a, C> {
    program: &'a Program<C>,
    store: Pubkey,
    deposit: Pubkey,
    cancel_for_user: Option<Pubkey>,
    execution_fee: u64,
    hint: Option<CancelDepositHint>,
}

impl<'a, S, C> CancelDepositBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(program: &'a Program<C>, store: &Pubkey, deposit: &Pubkey) -> Self {
        Self {
            program,
            store: *store,
            deposit: *deposit,
            cancel_for_user: None,
            execution_fee: 0,
            hint: None,
        }
    }

    /// Cancel for the given user.
    pub fn cancel_for_user(&mut self, user: &Pubkey) -> &mut Self {
        self.cancel_for_user = Some(*user);
        self
    }

    /// Set execution fee.
    pub fn execution_fee(&mut self, fee: u64) -> &mut Self {
        self.execution_fee = fee;
        self
    }

    /// Set hint with the given deposit.
    pub fn hint(&mut self, deposit: &Deposit) -> &mut Self {
        self.hint = Some(deposit.into());
        self
    }

    fn get_user_and_authority(&self) -> (Pubkey, Pubkey) {
        match self.cancel_for_user {
            Some(user) => (user, self.program.payer()),
            None => (
                self.program.payer(),
                ControllerSeeds::find_with_address(&self.store).1,
            ),
        }
    }

    async fn get_or_fetch_deposit_info(&self) -> crate::Result<CancelDepositHint> {
        match &self.hint {
            Some(hint) => Ok(*hint),
            None => {
                let deposit: Deposit = self.program.account(self.deposit).await?;
                Ok((&deposit).into())
            }
        }
    }

    /// Build a [`RequestBuilder`] for `cancel_deposit` instruction.
    pub async fn build(&self) -> crate::Result<RequestBuilder<'a, C>> {
        let (user, authority) = self.get_user_and_authority();
        let hint = self.get_or_fetch_deposit_info().await?;
        let Self {
            program,
            store,
            deposit,
            execution_fee,
            ..
        } = self;
        let only_controller = find_roles_address(store, &authority).0;
        Ok(program
            .request()
            .accounts(accounts::CancelDeposit {
                authority,
                store: *store,
                only_controller,
                data_store_program: data_store::id(),
                deposit: *deposit,
                user,
                initial_long_token: hint.initial_long_token_account,
                initial_short_token: hint.initial_short_token_account,
                long_token_deposit_vault: hint
                    .initial_long_token
                    .map(|token| find_market_vault_address(store, &token).0),
                short_token_deposit_vault: hint
                    .initial_short_token
                    .map(|token| find_market_vault_address(store, &token).0),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            })
            .args(instruction::CancelDeposit {
                execution_fee: *execution_fee,
            }))
    }
}

/// Execute Deposit Builder.
pub struct ExecuteDepositBuilder<'a, C> {
    program: &'a Program<C>,
    store: Pubkey,
    oracle: Pubkey,
    deposit: Pubkey,
    execution_fee: u64,
    hint: Option<ExecuteDepositHint>,
}

#[derive(Clone)]
struct ExecuteDepositHint {
    user: Pubkey,
    receiver: Pubkey,
    market_token_mint: Pubkey,
    feeds: Vec<Pubkey>,
    long_swap_tokens: Vec<Pubkey>,
    short_swap_tokens: Vec<Pubkey>,
}

impl<'a> From<&'a Deposit> for ExecuteDepositHint {
    fn from(deposit: &'a Deposit) -> Self {
        Self {
            user: deposit.fixed.senders.user,
            receiver: deposit.fixed.receivers.receiver,
            market_token_mint: deposit.fixed.tokens.market_token,
            feeds: deposit.dynamic.tokens_with_feed.feeds.clone(),
            long_swap_tokens: deposit.dynamic.swap_params.long_token_swap_path.clone(),
            short_swap_tokens: deposit.dynamic.swap_params.short_token_swap_path.clone(),
        }
    }
}

impl<'a, S, C> ExecuteDepositBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(
        program: &'a Program<C>,
        store: &Pubkey,
        oracle: &Pubkey,
        deposit: &Pubkey,
    ) -> Self {
        Self {
            program,
            store: *store,
            oracle: *oracle,
            deposit: *deposit,
            execution_fee: 0,
            hint: None,
        }
    }

    /// Set execution fee.
    pub fn execution_fee(&mut self, fee: u64) -> &mut Self {
        self.execution_fee = fee;
        self
    }

    /// Set hint with the given deposit.
    pub fn hint(&mut self, deposit: &Deposit) -> &mut Self {
        self.hint = Some(deposit.into());
        self
    }

    async fn get_or_fetch_hint(&self) -> crate::Result<ExecuteDepositHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let deposit: Deposit = self.program.account(self.deposit).await?;
                Ok((&deposit).into())
            }
        }
    }

    /// Build [`RequestBuilder`] for executing the deposit.
    pub async fn build(&self) -> crate::Result<RequestBuilder<'a, C>> {
        let hint = self.get_or_fetch_hint().await?;
        let Self {
            program,
            store,
            oracle,
            deposit,
            execution_fee,
            ..
        } = self;
        let authority = program.payer();
        let only_order_keeper = find_roles_address(store, &authority).0;
        let feeds = hint.feeds.iter().map(|pubkey| AccountMeta {
            pubkey: *pubkey,
            is_signer: false,
            is_writable: false,
        });
        let markets = hint
            .long_swap_tokens
            .iter()
            .chain(hint.short_swap_tokens.iter())
            .map(|mint| AccountMeta {
                pubkey: find_market_address(store, mint).0,
                is_signer: false,
                is_writable: true,
            });
        let market_tokens = hint
            .long_swap_tokens
            .iter()
            .chain(hint.short_swap_tokens.iter())
            .map(|mint| AccountMeta {
                pubkey: *mint,
                is_signer: false,
                is_writable: false,
            });
        Ok(program
            .request()
            .accounts(accounts::ExecuteDeposit {
                authority,
                only_order_keeper,
                store: *store,
                data_store_program: data_store::id(),
                chainlink_program: Chainlink::id(),
                token_program: anchor_spl::token::ID,
                oracle: *oracle,
                token_config_map: find_token_config_map(store).0,
                deposit: *deposit,
                user: hint.user,
                receiver: hint.receiver,
                market: find_market_address(store, &hint.market_token_mint).0,
                market_token_mint: hint.market_token_mint,
                system_program: system_program::ID,
            })
            .args(instruction::ExecuteDeposit {
                execution_fee: *execution_fee,
            })
            .accounts(
                feeds
                    .chain(markets)
                    .chain(market_tokens)
                    .collect::<Vec<_>>(),
            ))
    }
}
