use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use anchor_spl::associated_token::get_associated_token_address;
use data_store::states::{common::TokensWithFeed, Deposit, NonceBytes, Pyth};
use exchange::{accounts, instruction, instructions::CreateDepositParams};

use crate::{
    store::utils::{read_market, FeedsParser},
    utils::{ComputeBudget, RpcBuilder},
};

use super::generate_nonce;

#[cfg(feature = "pyth-pull-oracle")]
use crate::pyth::pull_oracle::Prices;

/// `execute_deposit` compute budget.
pub const EXECUTE_DEPOSIT_COMPUTE_BUDGET: u32 = 400_000;

/// Create Deposit Builder.
pub struct CreateDepositBuilder<'a, C> {
    client: &'a crate::Client<C>,
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
    token_map: Option<Pubkey>,
}

impl<'a, C, S> CreateDepositBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(client: &'a crate::Client<C>, store: Pubkey, market_token: Pubkey) -> Self {
        Self {
            client,
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
            token_map: None,
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
            None => Some(get_associated_token_address(&self.client.payer(), token)),
        }
    }

    fn get_or_find_associated_initial_short_token_account(
        &self,
        token: Option<&Pubkey>,
    ) -> Option<Pubkey> {
        let token = token?;
        match self.initial_short_token_account {
            Some(account) => Some(account),
            None => Some(get_associated_token_address(&self.client.payer(), token)),
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
                let market = read_market(&self.client.data_store().async_rpc(), market).await?;
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
                &self.client.payer(),
                &self.market_token,
            ),
        }
    }

    async fn get_token_map(&self) -> crate::Result<Pubkey> {
        if let Some(address) = self.token_map {
            Ok(address)
        } else {
            crate::store::utils::token_map(self.client.data_store(), &self.store).await
        }
    }

    /// Set token map address.
    pub fn token_map(&mut self, address: Pubkey) -> &mut Self {
        self.token_map = Some(address);
        self
    }

    /// Build a [`RequestBuilder`] and return deposit address.
    pub async fn build_with_address(&self) -> crate::Result<(RequestBuilder<'a, C>, Pubkey)> {
        let receiver = self.get_receiver();
        let Self {
            client,
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
        let payer = client.payer();
        let deposit = client.find_deposit_address(store, &payer, &nonce);
        let authority = client.controller_address(store);
        let market = client.find_market_address(store, market_token);
        let (long_token, short_token) = self.get_or_fetch_initial_tokens(&market).await?;
        let initial_long_token_account =
            self.get_or_find_associated_initial_long_token_account(long_token.as_ref());
        let initial_short_token_account =
            self.get_or_find_associated_initial_short_token_account(short_token.as_ref());
        let builder = client
            .exchange()
            .request()
            .accounts(crate::utils::fix_optional_account_metas(
                accounts::CreateDeposit {
                    authority,
                    store: *store,
                    data_store_program: client.data_store_program_id(),
                    system_program: system_program::ID,
                    token_program: anchor_spl::token::ID,
                    deposit,
                    payer,
                    receiver,
                    token_map: self.get_token_map().await?,
                    market,
                    initial_long_token_account,
                    initial_short_token_account,
                    initial_long_token_vault: long_token
                        .map(|token| client.find_market_vault_address(store, &token)),
                    initial_short_token_vault: short_token
                        .map(|token| client.find_market_vault_address(store, &token)),
                },
                &exchange::id(),
                &client.exchange_program_id(),
            ))
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
                    .enumerate()
                    .map(|(idx, mint)| AccountMeta {
                        pubkey: client.find_market_address(store, mint),
                        is_signer: false,
                        is_writable: idx == 0,
                    })
                    .chain(short_token_swap_path.iter().enumerate().map(|(idx, mint)| {
                        AccountMeta {
                            pubkey: client.find_market_address(store, mint),
                            is_signer: false,
                            is_writable: idx == 0,
                        }
                    }))
                    .collect::<Vec<_>>(),
            );
        Ok((builder, deposit))
    }
}

/// Cancel Deposit Builder.
pub struct CancelDepositBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    deposit: Pubkey,
    cancel_for_user: Option<Pubkey>,
    execution_fee: u64,
    hint: Option<CancelDepositHint>,
}

#[derive(Clone, Copy)]
struct CancelDepositHint {
    initial_long_token: Option<Pubkey>,
    initial_short_token: Option<Pubkey>,
    initial_long_token_account: Option<Pubkey>,
    initial_short_token_account: Option<Pubkey>,
    initial_long_market: Option<Pubkey>,
    initial_short_market: Option<Pubkey>,
}

impl<'a> CancelDepositHint {
    fn new(deposit: &'a Deposit, store_program_id: &Pubkey) -> Self {
        Self {
            initial_long_token: deposit.fixed.tokens.initial_long_token,
            initial_short_token: deposit.fixed.tokens.initial_short_token,
            initial_long_token_account: deposit.fixed.senders.initial_long_token_account,
            initial_short_token_account: deposit.fixed.senders.initial_short_token_account,
            initial_long_market: deposit.fixed.tokens.initial_long_token.map(|_| {
                crate::pda::find_market_address(
                    &deposit.fixed.store,
                    deposit
                        .dynamic
                        .swap_params
                        .first_market_token(true)
                        .unwrap_or(&deposit.fixed.tokens.market_token),
                    store_program_id,
                )
                .0
            }),
            initial_short_market: deposit.fixed.tokens.initial_short_token.map(|_| {
                crate::pda::find_market_address(
                    &deposit.fixed.store,
                    deposit
                        .dynamic
                        .swap_params
                        .first_market_token(false)
                        .unwrap_or(&deposit.fixed.tokens.market_token),
                    store_program_id,
                )
                .0
            }),
        }
    }
}

impl<'a, S, C> CancelDepositBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(client: &'a crate::Client<C>, store: &Pubkey, deposit: &Pubkey) -> Self {
        Self {
            client,
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

    /// Set execution fee to used
    pub fn execution_fee(&mut self, fee: u64) -> &mut Self {
        self.execution_fee = fee;
        self
    }

    /// Set hint with the given deposit.
    pub fn hint(&mut self, deposit: &Deposit) -> &mut Self {
        self.hint = Some(CancelDepositHint::new(
            deposit,
            &self.client.data_store_program_id(),
        ));
        self
    }

    fn get_user_and_authority(&self) -> (Pubkey, Pubkey) {
        match self.cancel_for_user {
            Some(user) => (user, self.client.payer()),
            None => (
                self.client.payer(),
                self.client.controller_address(&self.store),
            ),
        }
    }

    async fn get_or_fetch_deposit_info(&self) -> crate::Result<CancelDepositHint> {
        match &self.hint {
            Some(hint) => Ok(*hint),
            None => {
                let deposit: Deposit = self.client.data_store().account(self.deposit).await?;
                Ok(CancelDepositHint::new(
                    &deposit,
                    &self.client.data_store_program_id(),
                ))
            }
        }
    }

    /// Build a [`RequestBuilder`] for `cancel_deposit` instruction.
    pub async fn build(&self) -> crate::Result<RequestBuilder<'a, C>> {
        let (user, authority) = self.get_user_and_authority();
        let hint = self.get_or_fetch_deposit_info().await?;
        let Self {
            client,
            store,
            deposit,
            execution_fee,
            ..
        } = self;
        Ok(client
            .exchange()
            .request()
            .accounts(crate::utils::fix_optional_account_metas(
                accounts::CancelDeposit {
                    authority,
                    store: *store,
                    data_store_program: client.data_store_program_id(),
                    deposit: *deposit,
                    user,
                    initial_long_token: hint.initial_long_token_account,
                    initial_short_token: hint.initial_short_token_account,
                    long_token_deposit_vault: hint
                        .initial_long_token
                        .map(|token| client.find_market_vault_address(store, &token)),
                    short_token_deposit_vault: hint
                        .initial_short_token
                        .map(|token| client.find_market_vault_address(store, &token)),
                    initial_long_market: hint.initial_long_market,
                    initial_short_market: hint.initial_short_market,
                    token_program: anchor_spl::token::ID,
                    system_program: system_program::ID,
                },
                &exchange::id(),
                &client.exchange_program_id(),
            ))
            .args(instruction::CancelDeposit {
                execution_fee: *execution_fee,
            }))
    }
}

/// Execute Deposit Builder.
pub struct ExecuteDepositBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    oracle: Pubkey,
    deposit: Pubkey,
    execution_fee: u64,
    price_provider: Pubkey,
    feeds_parser: FeedsParser,
    hint: Option<ExecuteDepositHint>,
    token_map: Option<Pubkey>,
}

/// Hint for executing deposit.
#[derive(Clone)]
pub struct ExecuteDepositHint {
    user: Pubkey,
    receiver: Pubkey,
    market_token_mint: Pubkey,
    /// Feeds.
    pub feeds: TokensWithFeed,
    long_swap_tokens: Vec<Pubkey>,
    short_swap_tokens: Vec<Pubkey>,
}

impl<'a> From<&'a Deposit> for ExecuteDepositHint {
    fn from(deposit: &'a Deposit) -> Self {
        Self {
            user: deposit.fixed.senders.user,
            receiver: deposit.fixed.receivers.receiver,
            market_token_mint: deposit.fixed.tokens.market_token,
            feeds: deposit.dynamic.tokens_with_feed.clone(),
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
        client: &'a crate::Client<C>,
        store: &Pubkey,
        oracle: &Pubkey,
        deposit: &Pubkey,
    ) -> Self {
        Self {
            client,
            store: *store,
            oracle: *oracle,
            deposit: *deposit,
            execution_fee: 0,
            price_provider: Pyth::id(),
            hint: None,
            feeds_parser: Default::default(),
            token_map: None,
        }
    }

    /// Set execution fee.
    pub fn execution_fee(&mut self, fee: u64) -> &mut Self {
        self.execution_fee = fee;
        self
    }

    /// Set price provider to the given.
    pub fn price_provider(&mut self, program: Pubkey) -> &mut Self {
        self.price_provider = program;
        self
    }

    /// Set hint with the given deposit.
    pub fn hint(&mut self, deposit: &Deposit) -> &mut Self {
        self.hint = Some(deposit.into());
        self
    }

    /// Parse feeds with the given price udpates map.
    #[cfg(feature = "pyth-pull-oracle")]
    pub fn parse_with_pyth_price_updates(&mut self, price_updates: Prices) -> &mut Self {
        self.feeds_parser.with_pyth_price_updates(price_updates);
        self
    }

    /// Prepare [`ExecuteDepositHint`].
    pub async fn prepare_hint(&mut self) -> crate::Result<ExecuteDepositHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let deposit: Deposit = self.client.data_store().account(self.deposit).await?;
                let hint: ExecuteDepositHint = (&deposit).into();
                self.hint = Some(hint.clone());
                Ok(hint)
            }
        }
    }

    async fn get_token_map(&self) -> crate::Result<Pubkey> {
        if let Some(address) = self.token_map {
            Ok(address)
        } else {
            crate::store::utils::token_map(self.client.data_store(), &self.store).await
        }
    }

    /// Set token map.
    pub fn token_map(&mut self, address: Pubkey) -> &mut Self {
        self.token_map = Some(address);
        self
    }

    /// Build [`RpcBuilder`] for executing the deposit.
    pub async fn build(&mut self) -> crate::Result<RpcBuilder<'a, C>> {
        let token_map = self.get_token_map().await?;
        let hint = self.prepare_hint().await?;
        let Self {
            client,
            store,
            oracle,
            deposit,
            execution_fee,
            price_provider,
            ..
        } = self;
        let authority = client.payer();
        let feeds = self
            .feeds_parser
            .parse(&hint.feeds)
            .collect::<Result<Vec<_>, _>>()?;
        let markets = hint
            .long_swap_tokens
            .iter()
            .chain(hint.short_swap_tokens.iter())
            .map(|mint| AccountMeta {
                pubkey: client.find_market_address(store, mint),
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
        tracing::debug!(%price_provider, "constructing `execute_deposit` ix...");
        Ok(client
            .exchange_request()
            .accounts(accounts::ExecuteDeposit {
                authority,
                controller: client.controller_address(store),
                store: *store,
                data_store_program: client.data_store_program_id(),
                price_provider: *price_provider,
                token_program: anchor_spl::token::ID,
                oracle: *oracle,
                token_map,
                deposit: *deposit,
                user: hint.user,
                receiver: hint.receiver,
                market: client.find_market_address(store, &hint.market_token_mint),
                market_token_mint: hint.market_token_mint,
                system_program: system_program::ID,
            })
            .args(instruction::ExecuteDeposit {
                execution_fee: *execution_fee,
            })
            .accounts(
                feeds
                    .into_iter()
                    .chain(markets)
                    .chain(market_tokens)
                    .collect::<Vec<_>>(),
            )
            .compute_budget(ComputeBudget::default().with_limit(EXECUTE_DEPOSIT_COMPUTE_BUDGET)))
    }
}
