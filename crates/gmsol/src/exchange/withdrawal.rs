use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use anchor_spl::associated_token::get_associated_token_address;
use data_store::states::{
    common::{SwapParams, TokensWithFeed},
    withdrawal::TokenParams,
    NonceBytes, Pyth, Withdrawal,
};
use exchange::{accounts, instruction, instructions::CreateWithdrawalParams};

use crate::{
    store::utils::{read_market, FeedsParser},
    utils::{ComputeBudget, RpcBuilder},
};

use super::generate_nonce;

#[cfg(feature = "pyth-pull-oracle")]
use crate::pyth::pull_oracle::Prices;

/// `execute_withdrawal` compute budget.
pub const EXECUTE_WITHDRAWAL_COMPUTE_BUDGET: u32 = 400_000;

/// Create Withdrawal Builder.
pub struct CreateWithdrawalBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    market_token: Pubkey,
    nonce: Option<NonceBytes>,
    execution_fee: u64,
    amount: u64,
    ui_fee_receiver: Pubkey,
    min_long_token_amount: u64,
    min_short_token_amount: u64,
    should_unwrap_native_token: bool,
    market_token_account: Option<Pubkey>,
    final_long_token: Option<Pubkey>,
    final_short_token: Option<Pubkey>,
    final_long_token_receiver: Option<Pubkey>,
    final_short_token_receiver: Option<Pubkey>,
    long_token_swap_path: Vec<Pubkey>,
    short_token_swap_path: Vec<Pubkey>,
    token_map: Option<Pubkey>,
}

impl<'a, C, S> CreateWithdrawalBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: Pubkey,
        market_token: Pubkey,
        amount: u64,
    ) -> Self {
        Self {
            client,
            store,
            market_token,
            nonce: None,
            execution_fee: 0,
            amount,
            ui_fee_receiver: Pubkey::new_unique(),
            min_long_token_amount: 0,
            min_short_token_amount: 0,
            should_unwrap_native_token: false,
            market_token_account: None,
            final_long_token: None,
            final_short_token: None,
            final_long_token_receiver: None,
            final_short_token_receiver: None,
            long_token_swap_path: vec![],
            short_token_swap_path: vec![],
            token_map: None,
        }
    }

    /// Set extra exectuion fee allowed to use.
    ///
    /// Defaults to `0` means only allowed to use at most `rent-exempt` amount of fee.
    pub fn execution_fee(&mut self, amount: u64) -> &mut Self {
        self.execution_fee = amount;
        self
    }

    /// Set min final long token amount.
    pub fn min_final_long_token_amount(&mut self, amount: u64) -> &mut Self {
        self.min_long_token_amount = amount;
        self
    }

    /// Set min final short token amount.
    pub fn min_final_short_token_amount(&mut self, amount: u64) -> &mut Self {
        self.min_short_token_amount = amount;
        self
    }

    /// Set market token source account to the given.
    pub fn market_token_account(&mut self, account: &Pubkey) -> &mut Self {
        self.market_token_account = Some(*account);
        self
    }

    /// Set final long token params.
    pub fn final_long_token(
        &mut self,
        token: &Pubkey,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.final_long_token = Some(*token);
        self.final_long_token_receiver = token_account.copied();
        self
    }

    /// Set final short token params.
    pub fn final_short_token(
        &mut self,
        token: &Pubkey,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.final_short_token = Some(*token);
        self.final_short_token_receiver = token_account.copied();
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

    fn get_or_find_associated_market_token_account(&self) -> Pubkey {
        match self.market_token_account {
            Some(account) => account,
            None => get_associated_token_address(&self.client.payer(), &self.market_token),
        }
    }

    fn get_or_find_associated_final_long_token_account(&self, token: &Pubkey) -> Pubkey {
        match self.final_long_token_receiver {
            Some(account) => account,
            None => get_associated_token_address(&self.client.payer(), token),
        }
    }

    fn get_or_find_associated_final_short_token_account(&self, token: &Pubkey) -> Pubkey {
        match self.final_short_token_receiver {
            Some(account) => account,
            None => get_associated_token_address(&self.client.payer(), token),
        }
    }

    async fn get_or_fetch_final_tokens(&self, market: &Pubkey) -> crate::Result<(Pubkey, Pubkey)> {
        if let (Some(long_token), Some(short_token)) =
            (self.final_long_token, self.final_short_token)
        {
            return Ok((long_token, short_token));
        }
        let market = read_market(&self.client.data_store().async_rpc(), market).await?;
        Ok((
            self.final_long_token
                .unwrap_or_else(|| market.meta().long_token_mint),
            self.final_short_token
                .unwrap_or_else(|| market.meta().short_token_mint),
        ))
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

    /// Create the [`RequestBuilder`] and return withdrawal address.
    pub async fn build_with_address(&self) -> crate::Result<(RequestBuilder<'a, C>, Pubkey)> {
        let payer = self.client.payer();
        let authority = self.client.controller_address(&self.store);
        let market_token_withdrawal_vault = self
            .client
            .find_market_vault_address(&self.store, &self.market_token);
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let withdrawal = self
            .client
            .find_withdrawal_address(&self.store, &payer, &nonce);
        let market = self
            .client
            .find_market_address(&self.store, &self.market_token);
        let (long_token, short_token) = self.get_or_fetch_final_tokens(&market).await?;
        let builder = self
            .client
            .exchange()
            .request()
            .accounts(accounts::CreateWithdrawal {
                authority,
                store: self.store,
                data_store_program: self.client.data_store_program_id(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                token_map: self.get_token_map().await?,
                market,
                withdrawal,
                payer,
                market_token_account: self.get_or_find_associated_market_token_account(),
                market_token_withdrawal_vault,
                final_long_token_receiver: self
                    .get_or_find_associated_final_long_token_account(&long_token),
                final_short_token_receiver: self
                    .get_or_find_associated_final_short_token_account(&short_token),
            })
            .args(instruction::CreateWithdrawal {
                nonce,
                params: CreateWithdrawalParams {
                    market_token_amount: self.amount,
                    execution_fee: self.execution_fee,
                    ui_fee_receiver: self.ui_fee_receiver,
                    tokens: TokenParams {
                        min_long_token_amount: self.min_long_token_amount,
                        min_short_token_amount: self.min_short_token_amount,
                        should_unwrap_native_token: self.should_unwrap_native_token,
                    },
                    long_token_swap_length: self
                        .long_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::NumberOutOfRange)?,
                    short_token_swap_length: self
                        .short_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::NumberOutOfRange)?,
                },
            })
            .accounts(
                self.long_token_swap_path
                    .iter()
                    .chain(self.short_token_swap_path.iter())
                    .map(|mint| AccountMeta {
                        pubkey: self.client.find_market_address(&self.store, mint),
                        is_signer: false,
                        is_writable: false,
                    })
                    .collect::<Vec<_>>(),
            );

        Ok((builder, withdrawal))
    }
}

/// Cancel Withdrawal Builder.
pub struct CancelWithdrawalBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    withdrawal: Pubkey,
    hint: Option<CancelWithdrawalHint>,
}

#[derive(Clone, Copy)]
struct CancelWithdrawalHint {
    market_token: Pubkey,
    market_token_account: Pubkey,
}

impl<'a> From<&'a Withdrawal> for CancelWithdrawalHint {
    fn from(withdrawal: &'a Withdrawal) -> Self {
        Self {
            market_token: withdrawal.fixed.tokens.market_token,
            market_token_account: withdrawal.fixed.market_token_account,
        }
    }
}

impl<'a, S, C> CancelWithdrawalBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(client: &'a crate::Client<C>, store: &Pubkey, withdrawal: &Pubkey) -> Self {
        Self {
            client,
            store: *store,
            withdrawal: *withdrawal,
            hint: None,
        }
    }

    /// Set hint with the given withdrawal.
    pub fn hint(&mut self, withdrawal: &Withdrawal) -> &mut Self {
        self.hint = Some(withdrawal.into());
        self
    }

    async fn get_or_fetch_withdrawal_hint(&self) -> crate::Result<CancelWithdrawalHint> {
        match &self.hint {
            Some(hint) => Ok(*hint),
            None => {
                let withdrawal: Withdrawal =
                    self.client.data_store().account(self.withdrawal).await?;
                Ok((&withdrawal).into())
            }
        }
    }

    /// Build a [`RequestBuilder`] for `cancel_withdrawal` instruction.
    pub async fn build(&self) -> crate::Result<RequestBuilder<'a, C>> {
        let user = self.client.payer();
        let hint = self.get_or_fetch_withdrawal_hint().await?;
        Ok(self
            .client
            .exchange()
            .request()
            .accounts(accounts::CancelWithdrawal {
                user,
                controller: self.client.controller_address(&self.store),
                store: self.store,
                withdrawal: self.withdrawal,
                market_token: hint.market_token_account,
                market_token_withdrawal_vault: self
                    .client
                    .find_market_vault_address(&self.store, &hint.market_token),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                event_authority: self.client.data_store_event_authority(),
                data_store_program: self.client.data_store_program_id(),
            })
            .args(instruction::CancelWithdrawal {}))
    }
}

/// Execute Withdrawal Builder.
pub struct ExecuteWithdrawalBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    oracle: Pubkey,
    withdrawal: Pubkey,
    execution_fee: u64,
    price_provider: Pubkey,
    hint: Option<ExecuteWithdrawalHint>,
    feeds_parser: FeedsParser,
    token_map: Option<Pubkey>,
    cancel_on_execution_error: bool,
}

/// Hint for withdrawal execution.
#[derive(Clone)]
pub struct ExecuteWithdrawalHint {
    market_token: Pubkey,
    user: Pubkey,
    market_token_account: Pubkey,
    final_long_token_receiver: Pubkey,
    final_short_token_receiver: Pubkey,
    final_long_token: Pubkey,
    final_short_token: Pubkey,
    /// Feeds.
    pub feeds: TokensWithFeed,
    swap: SwapParams,
}

impl<'a> From<&'a Withdrawal> for ExecuteWithdrawalHint {
    fn from(withdrawal: &'a Withdrawal) -> Self {
        Self {
            market_token: withdrawal.fixed.tokens.market_token,
            user: withdrawal.fixed.user,
            market_token_account: withdrawal.fixed.market_token_account,
            final_long_token: withdrawal.fixed.tokens.final_long_token,
            final_short_token: withdrawal.fixed.tokens.final_short_token,
            final_long_token_receiver: withdrawal.fixed.receivers.final_long_token_receiver,
            final_short_token_receiver: withdrawal.fixed.receivers.final_short_token_receiver,
            swap: withdrawal.dynamic.swap.clone(),
            feeds: withdrawal.dynamic.tokens_with_feed.clone(),
        }
    }
}

impl<'a, S, C> ExecuteWithdrawalBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        oracle: &Pubkey,
        withdrawal: &Pubkey,
        cancel_on_execution_error: bool,
    ) -> Self {
        Self {
            client,
            store: *store,
            oracle: *oracle,
            withdrawal: *withdrawal,
            execution_fee: 0,
            price_provider: Pyth::id(),
            hint: None,
            feeds_parser: Default::default(),
            token_map: None,
            cancel_on_execution_error,
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

    /// Set hint with the given withdrawal.
    pub fn hint(&mut self, withdrawal: &Withdrawal) -> &mut Self {
        self.hint = Some(withdrawal.into());
        self
    }

    /// Parse feeds with the given price udpates map.
    #[cfg(feature = "pyth-pull-oracle")]
    pub fn parse_with_pyth_price_updates(&mut self, price_updates: Prices) -> &mut Self {
        self.feeds_parser.with_pyth_price_updates(price_updates);
        self
    }

    /// Prepare [`ExecuteWithdrawalHint`].
    pub async fn prepare_hint(&mut self) -> crate::Result<ExecuteWithdrawalHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let withdrawal: Withdrawal =
                    self.client.data_store().account(self.withdrawal).await?;
                let hint: ExecuteWithdrawalHint = (&withdrawal).into();
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

    /// Build [`RpcBuilder`] for `execute_withdrawal` instruction.
    pub async fn build(&mut self) -> crate::Result<RpcBuilder<'a, C>> {
        let authority = self.client.payer();
        let hint = self.prepare_hint().await?;
        let feeds = self
            .feeds_parser
            .parse(&hint.feeds)
            .collect::<Result<Vec<_>, _>>()?;
        let swap_path_markets = hint
            .swap
            .unique_market_tokens_excluding_current(&hint.market_token)
            .map(|mint| AccountMeta {
                pubkey: self.client.find_market_address(&self.store, mint),
                is_signer: false,
                is_writable: true,
            });
        Ok(self
            .client
            .exchange_rpc()
            .accounts(accounts::ExecuteWithdrawal {
                authority,
                store: self.store,
                controller: self.client.controller_address(&self.store),
                event_authority: self.client.data_store_event_authority(),
                data_store_program: self.client.data_store_program_id(),
                price_provider: self.price_provider,
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                oracle: self.oracle,
                token_map: self.get_token_map().await?,
                withdrawal: self.withdrawal,
                market: self
                    .client
                    .find_market_address(&self.store, &hint.market_token),
                user: hint.user,
                market_token_mint: hint.market_token,
                market_token_withdrawal_vault: self
                    .client
                    .find_market_vault_address(&self.store, &hint.market_token),
                market_token_account: hint.market_token_account,
                final_long_token_receiver: hint.final_long_token_receiver,
                final_short_token_receiver: hint.final_short_token_receiver,
                final_long_token_vault: self
                    .client
                    .find_market_vault_address(&self.store, &hint.final_long_token),
                final_short_token_vault: self
                    .client
                    .find_market_vault_address(&self.store, &hint.final_short_token),
            })
            .args(instruction::ExecuteWithdrawal {
                execution_fee: self.execution_fee,
                cancel_on_execution_error: self.cancel_on_execution_error,
            })
            .accounts(
                feeds
                    .into_iter()
                    .chain(swap_path_markets)
                    .collect::<Vec<_>>(),
            )
            .compute_budget(ComputeBudget::default().with_limit(EXECUTE_WITHDRAWAL_COMPUTE_BUDGET)))
    }
}
