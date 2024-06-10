use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use anchor_spl::associated_token::get_associated_token_address;
use data_store::states::{
    common::{SwapParams, TokensWithFeed},
    order::{OrderKind, OrderParams},
    Config, Market, MarketMeta, NonceBytes, Order, Pyth,
};
use exchange::{accounts, instruction, instructions::CreateOrderParams};

use crate::{
    store::utils::FeedsParser,
    utils::{ComputeBudget, TokenAccountParams, TransactionBuilder},
};

use super::generate_nonce;

#[cfg(feature = "pyth-pull-oracle")]
use crate::pyth::pull_oracle::Prices;

/// `execute_order` compute budget.
pub const EXECUTE_ORDER_COMPUTE_BUDGET: u32 = 400_000;

/// Create Order Builder.
pub struct CreateOrderBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    market_token: Pubkey,
    is_output_token_long: bool,
    nonce: Option<NonceBytes>,
    execution_fee: u64,
    ui_fee_receiver: Pubkey,
    params: OrderParams,
    swap_path: Vec<Pubkey>,
    hint: Option<CreateOrderHint>,
    initial_token: TokenAccountParams,
    final_token: TokenAccountParams,
    secondary_token_account: Option<Pubkey>,
    long_token_account: Option<Pubkey>,
    short_token_account: Option<Pubkey>,
    token_map: Option<Pubkey>,
}

#[derive(Clone, Copy)]
struct CreateOrderHint {
    long_token: Pubkey,
    short_token: Pubkey,
}

impl<'a, C, S> CreateOrderBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        market_token: &Pubkey,
        params: OrderParams,
        is_output_token_long: bool,
    ) -> Self {
        Self {
            client,
            store: *store,
            market_token: *market_token,
            nonce: None,
            execution_fee: 0,
            ui_fee_receiver: Pubkey::new_unique(),
            params,
            swap_path: vec![],
            is_output_token_long,
            hint: None,
            initial_token: Default::default(),
            final_token: Default::default(),
            secondary_token_account: None,
            long_token_account: None,
            short_token_account: None,
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

    /// Setup hint with the given market meta.
    pub fn hint(&mut self, meta: &MarketMeta) -> &mut Self {
        self.hint = Some(CreateOrderHint {
            long_token: meta.long_token_mint,
            short_token: meta.short_token_mint,
        });
        self
    }

    /// Set swap path.
    pub fn swap_path(&mut self, market_tokens: Vec<Pubkey>) -> &mut Self {
        self.swap_path = market_tokens;
        self
    }

    /// Set initial collateral token (or swap-in token) params.
    pub fn initial_collateral_token(
        &mut self,
        token: &Pubkey,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.initial_token.set_token(*token);
        if let Some(account) = token_account {
            self.initial_token.set_token_account(*account);
        }
        self
    }

    /// Set final output token (or swap-out token) params.
    pub fn final_output_token(
        &mut self,
        token: &Pubkey,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.final_token.set_token(*token);
        if let Some(account) = token_account {
            self.final_token.set_token_account(*account);
        }
        self
    }

    /// Set secondary output token account.
    pub fn secondary_output_token_account(&mut self, account: &Pubkey) -> &mut Self {
        self.secondary_token_account = Some(*account);
        self
    }

    /// Set long token account.
    pub fn long_token_account(&mut self, account: &Pubkey) -> &mut Self {
        self.long_token_account = Some(*account);
        self
    }

    /// Set short token account.
    pub fn short_token_account(&mut self, account: &Pubkey) -> &mut Self {
        self.short_token_account = Some(*account);
        self
    }

    /// Set min output amount.
    pub fn min_output_amount(&mut self, amount: u64) -> &mut Self {
        self.params.min_output_amount = amount;
        self
    }

    /// Set acceptable price.
    pub fn acceptable_price(&mut self, unit_price: u128) -> &mut Self {
        self.params.acceptable_price = Some(unit_price);
        self
    }

    fn market(&self) -> Pubkey {
        self.client
            .find_market_address(&self.store, &self.market_token)
    }

    async fn prepare_hint(&mut self) -> crate::Result<CreateOrderHint> {
        loop {
            if let Some(hint) = self.hint {
                return Ok(hint);
            }
            let market = self
                .client
                .data_store()
                .account::<Market>(self.market())
                .await?;
            self.hint(market.meta());
        }
    }

    async fn output_token(&mut self) -> crate::Result<Pubkey> {
        let hint = self.prepare_hint().await?;
        let output_token = if self.is_output_token_long {
            hint.long_token
        } else {
            hint.short_token
        };
        Ok(output_token)
    }

    async fn output_token_and_position(&mut self) -> crate::Result<(Pubkey, Option<Pubkey>)> {
        let output_token = self.output_token().await?;
        match &self.params.kind {
            OrderKind::MarketIncrease | OrderKind::MarketDecrease | OrderKind::Liquidation => {
                let position = self.client.find_position_address(
                    &self.store,
                    &self.client.payer(),
                    &self.market_token,
                    &output_token,
                    self.params
                        .to_position_kind()
                        .map_err(anchor_client::ClientError::from)?,
                )?;
                Ok((output_token, Some(position)))
            }
            OrderKind::MarketSwap => Ok((output_token, None)),
            kind => Err(crate::Error::invalid_argument(format!(
                "unsupported order kind: {kind:?}"
            ))),
        }
    }

    /// Get initial collateral token account and vault.
    ///
    /// Returns `(initial_collateral_token_account, initial_collateral_token_vault)`.
    async fn initial_collateral_accounts(&mut self) -> crate::Result<Option<(Pubkey, Pubkey)>> {
        match &self.params.kind {
            OrderKind::MarketIncrease | OrderKind::MarketSwap => {
                if self.initial_token.is_empty() {
                    let output_token = self.output_token().await?;
                    self.initial_token.set_token(output_token);
                }
                let Some((token, account)) = self
                    .initial_token
                    .get_or_fetch_token_and_token_account(
                        self.client.exchange(),
                        Some(&self.client.payer()),
                    )
                    .await?
                else {
                    return Err(crate::Error::invalid_argument(
                        "missing initial collateral token parameters",
                    ));
                };
                Ok(Some((
                    account,
                    self.client.find_market_vault_address(&self.store, &token),
                )))
            }
            OrderKind::MarketDecrease | OrderKind::Liquidation => Ok(None),
            kind => Err(crate::Error::invalid_argument(format!(
                "unsupported order kind: {kind:?}"
            ))),
        }
    }

    async fn final_output_token_account(&mut self) -> crate::Result<Option<Pubkey>> {
        match &self.params.kind {
            OrderKind::MarketSwap | OrderKind::MarketDecrease | OrderKind::Liquidation => {
                if self.final_token.is_empty() {
                    let output_token = self.output_token().await?;
                    self.final_token.set_token(output_token);
                }
                let Some(account) = self
                    .final_token
                    .get_or_find_associated_token_account(Some(&self.client.payer()))
                else {
                    return Err(crate::Error::invalid_argument(
                        "missing final output token parameters",
                    ));
                };
                Ok(Some(account))
            }
            OrderKind::MarketIncrease => Ok(None),
            kind => Err(crate::Error::invalid_argument(format!(
                "unsupported order kind: {kind:?}"
            ))),
        }
    }

    async fn secondary_output_token(&mut self) -> crate::Result<Pubkey> {
        let hint = self.prepare_hint().await?;
        Ok(if self.params.is_long {
            hint.long_token
        } else {
            hint.short_token
        })
    }

    async fn get_secondary_output_token_account(&mut self) -> crate::Result<Option<Pubkey>> {
        match &self.params.kind {
            OrderKind::MarketIncrease | OrderKind::MarketSwap => Ok(None),
            OrderKind::MarketDecrease | OrderKind::Liquidation => {
                if let Some(account) = self.secondary_token_account {
                    return Ok(Some(account));
                }
                let secondary_output_token = self.secondary_output_token().await?;
                Ok(TokenAccountParams::default()
                    .set_token(secondary_output_token)
                    .get_or_find_associated_token_account(Some(&self.client.payer())))
            }
            kind => Err(crate::Error::invalid_argument(format!(
                "unsupported order kind: {kind:?}"
            ))),
        }
    }

    async fn collateral_token_accounts(&mut self) -> crate::Result<(Pubkey, Pubkey)> {
        let hint = self.prepare_hint().await?;
        let payer = self.client.payer();
        let long_token_account = self
            .long_token_account
            .unwrap_or(get_associated_token_address(&payer, &hint.long_token));
        let short_token_account = self
            .short_token_account
            .unwrap_or(get_associated_token_address(&payer, &hint.short_token));
        Ok((long_token_account, short_token_account))
    }

    async fn token_map(&self) -> crate::Result<Pubkey> {
        if let Some(address) = self.token_map {
            Ok(address)
        } else {
            crate::store::utils::token_map(self.client.data_store(), &self.store).await
        }
    }

    /// Create [`RequestBuilder`] and return order address.
    pub async fn build_with_address(&mut self) -> crate::Result<(RequestBuilder<'a, C>, Pubkey)> {
        let authority = self.client.controller_address(&self.store);
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let payer = &self.client.payer();
        let order = self.client.find_order_address(&self.store, payer, &nonce);
        let (initial_collateral_token_account, initial_collateral_token_vault) =
            self.initial_collateral_accounts().await?.unzip();
        let (output_token, position) = self.output_token_and_position().await?;
        let (long_token_account, short_token_account) = self.collateral_token_accounts().await?;
        let need_to_transfer_in = self.params.need_to_transfer_in();
        let builder = self
            .client
            .exchange()
            .request()
            .accounts(crate::utils::fix_optional_account_metas(
                accounts::CreateOrder {
                    authority,
                    store: self.store,
                    payer: *payer,
                    order,
                    position,
                    token_map: self.token_map().await?,
                    market: self.market(),
                    initial_collateral_token_account,
                    final_output_token_account: self.final_output_token_account().await?,
                    secondary_output_token_account: self
                        .get_secondary_output_token_account()
                        .await?,
                    initial_collateral_token_vault,
                    data_store_program: self.client.data_store_program_id(),
                    long_token_account,
                    short_token_account,
                    system_program: system_program::ID,
                    token_program: anchor_spl::token::ID,
                },
                &exchange::id(),
                &self.client.exchange_program_id(),
            ))
            .args(instruction::CreateOrder {
                nonce,
                params: CreateOrderParams {
                    order: self.params.clone(),
                    output_token,
                    ui_fee_receiver: self.ui_fee_receiver,
                    execution_fee: self.execution_fee,
                    swap_length: self
                        .swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::NumberOutOfRange)?,
                },
            })
            .accounts(
                self.swap_path
                    .iter()
                    .enumerate()
                    .map(|(idx, mint)| AccountMeta {
                        pubkey: self.client.find_market_address(&self.store, mint),
                        is_signer: false,
                        is_writable: need_to_transfer_in && idx == 0,
                    })
                    .collect::<Vec<_>>(),
            );

        Ok((builder, order))
    }
}

/// Execute Order Builder.
pub struct ExecuteOrderBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    oracle: Pubkey,
    order: Pubkey,
    execution_fee: u64,
    price_provider: Pubkey,
    feeds_parser: FeedsParser,
    recent_timestamp: i64,
    hint: Option<ExecuteOrderHint>,
    token_map: Option<Pubkey>,
}

/// Hint for executing order.
#[derive(Clone)]
pub struct ExecuteOrderHint {
    store_program_id: Pubkey,
    has_claimable: bool,
    config: Config,
    market_token: Pubkey,
    position: Option<Pubkey>,
    user: Pubkey,
    final_output_token: Option<Pubkey>,
    secondary_output_token: Pubkey,
    final_output_token_account: Option<Pubkey>,
    secondary_output_token_account: Option<Pubkey>,
    long_token_account: Pubkey,
    short_token_account: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
    pnl_token_mint: Pubkey,
    /// Feeds.
    pub feeds: TokensWithFeed,
    swap: SwapParams,
}

impl ExecuteOrderHint {
    fn long_token_vault(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_market_vault_address(store, &self.long_token_mint, &self.store_program_id)
            .0
    }

    fn short_token_vault(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_market_vault_address(store, &self.short_token_mint, &self.store_program_id)
            .0
    }

    fn claimable_long_token_account(
        &self,
        store: &Pubkey,
        timestamp: i64,
    ) -> crate::Result<Option<Pubkey>> {
        if !self.has_claimable {
            return Ok(None);
        }
        Ok(Some(
            crate::pda::find_claimable_account_pda(
                store,
                &self.long_token_mint,
                &self.user,
                &self.config.claimable_time_key(timestamp)?,
                &self.store_program_id,
            )
            .0,
        ))
    }

    fn claimable_short_token_account(
        &self,
        store: &Pubkey,
        timestamp: i64,
    ) -> crate::Result<Option<Pubkey>> {
        if !self.has_claimable {
            return Ok(None);
        }
        Ok(Some(
            crate::pda::find_claimable_account_pda(
                store,
                &self.short_token_mint,
                &self.user,
                &self.config.claimable_time_key(timestamp)?,
                &self.store_program_id,
            )
            .0,
        ))
    }

    fn claimable_pnl_token_account_for_holding(
        &self,
        store: &Pubkey,
        timestamp: i64,
    ) -> crate::Result<Option<Pubkey>> {
        if !self.has_claimable {
            return Ok(None);
        }
        Ok(Some(
            crate::pda::find_claimable_account_pda(
                store,
                &self.pnl_token_mint,
                &self.config.holding()?,
                &self.config.claimable_time_key(timestamp)?,
                &self.store_program_id,
            )
            .0,
        ))
    }
}

impl<'a, S, C> ExecuteOrderBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn try_new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        oracle: &Pubkey,
        order: &Pubkey,
    ) -> crate::Result<Self> {
        use std::time::SystemTime;

        let recent_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(crate::Error::unknown)?
            .as_secs()
            .try_into()
            .map_err(|_| crate::Error::unknown("failed to convert timestamp"))?;
        Ok(Self {
            client,
            store: *store,
            oracle: *oracle,
            order: *order,
            execution_fee: 0,
            price_provider: Pyth::id(),
            feeds_parser: Default::default(),
            recent_timestamp,
            hint: None,
            token_map: None,
        })
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

    /// Set hint with the given order.
    pub fn hint(&mut self, order: &Order, market: &Market, config: &Config) -> &mut Self {
        let swap = order.swap.clone();
        let market_token = order.fixed.tokens.market_token;
        let final_output_token_account = order.fixed.receivers.final_output_token_account;
        let secondary_output_token_account = order.fixed.receivers.secondary_output_token_account;
        self.hint = Some(ExecuteOrderHint {
            store_program_id: self.client.data_store_program_id(),
            has_claimable: matches!(order.fixed.params.kind, OrderKind::MarketDecrease),
            config: config.clone(),
            market_token,
            position: order.fixed.position,
            user: order.fixed.user,
            final_output_token: order.fixed.tokens.final_output_token,
            secondary_output_token: order.fixed.tokens.secondary_output_token,
            final_output_token_account,
            secondary_output_token_account,
            long_token_account: order.fixed.receivers.long_token_account,
            short_token_account: order.fixed.receivers.short_token_account,
            long_token_mint: market.meta().long_token_mint,
            short_token_mint: market.meta().short_token_mint,
            pnl_token_mint: if order.fixed.params.is_long {
                market.meta().long_token_mint
            } else {
                market.meta().short_token_mint
            },
            feeds: order.prices.clone(),
            swap,
        });
        self
    }

    /// Parse feeds with the given price udpates map.
    #[cfg(feature = "pyth-pull-oracle")]
    pub fn parse_with_pyth_price_updates(&mut self, price_updates: Prices) -> &mut Self {
        self.feeds_parser.with_pyth_price_updates(price_updates);
        self
    }

    /// Prepare [`ExecuteOrderHint`].
    pub async fn prepare_hint(&mut self) -> crate::Result<ExecuteOrderHint> {
        loop {
            match &self.hint {
                Some(hint) => return Ok(hint.clone()),
                None => {
                    let order: Order = self.client.data_store().account(self.order).await?;
                    let market: Market =
                        self.client.data_store().account(order.fixed.market).await?;
                    let config: Config = self
                        .client
                        .data_store()
                        .account(self.config_address())
                        .await?;
                    self.hint(&order, &market, &config);
                }
            }
        }
    }

    /// Set recent timestamp with the given.
    ///
    /// Default to current unix timestamp.
    pub fn recent_timestamp(&mut self, timestamp: i64) -> &mut Self {
        self.recent_timestamp = timestamp;
        self
    }

    /// Get claimable accounts.
    ///
    /// The returned values are of the form `[long_for_user, short_for_user, pnl_for_holding]`.
    pub async fn claimable_accounts(&mut self) -> crate::Result<[Option<Pubkey>; 3]> {
        let hint = self.prepare_hint().await?;
        let long_for_user =
            hint.claimable_long_token_account(&self.store, self.recent_timestamp)?;
        let short_for_user =
            hint.claimable_short_token_account(&self.store, self.recent_timestamp)?;
        let pnl_for_holding =
            hint.claimable_pnl_token_account_for_holding(&self.store, self.recent_timestamp)?;
        Ok([long_for_user, short_for_user, pnl_for_holding])
    }

    fn config_address(&self) -> Pubkey {
        self.client.find_config_address(&self.store)
    }

    async fn token_map(&self) -> crate::Result<Pubkey> {
        if let Some(address) = self.token_map {
            Ok(address)
        } else {
            crate::store::utils::token_map(self.client.data_store(), &self.store).await
        }
    }

    /// Build [`TransactionBuilder`] for `execute_order` instructions.
    pub async fn build(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        use crate::store::token::TokenAccountOps;

        let hint = self.prepare_hint().await?;
        let [claimable_long_token_account_for_user, claimable_short_token_account_for_user, claimable_pnl_token_account_for_holding] =
            self.claimable_accounts().await?;
        let authority = self.client.payer();
        let feeds = self
            .feeds_parser
            .parse(&hint.feeds)
            .collect::<Result<Vec<_>, _>>()?;
        let swap_markets = hint.swap.iter().map(|mint| AccountMeta {
            pubkey: self.client.find_market_address(&self.store, mint),
            is_signer: false,
            is_writable: true,
        });
        let swap_market_mints = hint.swap.iter().map(|pubkey| AccountMeta {
            pubkey: *pubkey,
            is_signer: false,
            is_writable: false,
        });

        let execute_order = self
            .client
            .exchange_request()
            .accounts(crate::utils::fix_optional_account_metas(
                accounts::ExecuteOrder {
                    authority,
                    controller: self.client.controller_address(&self.store),
                    store: self.store,
                    oracle: self.oracle,
                    config: self.config_address(),
                    token_config_map: self.token_map().await?,
                    market: self
                        .client
                        .find_market_address(&self.store, &hint.market_token),
                    market_token_mint: hint.market_token,
                    order: self.order,
                    position: hint.position,
                    user: hint.user,
                    final_output_token_vault: hint
                        .final_output_token_account
                        .as_ref()
                        .and(hint.final_output_token.as_ref())
                        .map(|token| self.client.find_market_vault_address(&self.store, token)),
                    secondary_output_token_vault: hint.secondary_output_token_account.as_ref().map(
                        |_| {
                            self.client.find_market_vault_address(
                                &self.store,
                                &hint.secondary_output_token,
                            )
                        },
                    ),
                    final_output_token_account: hint.final_output_token_account,
                    secondary_output_token_account: hint.secondary_output_token_account,
                    long_token_vault: hint.long_token_vault(&self.store),
                    short_token_vault: hint.short_token_vault(&self.store),
                    long_token_account: hint.long_token_account,
                    short_token_account: hint.short_token_account,
                    claimable_long_token_account_for_user,
                    claimable_short_token_account_for_user,
                    claimable_pnl_token_account_for_holding,
                    data_store_program: self.client.data_store_program_id(),
                    token_program: anchor_spl::token::ID,
                    price_provider: self.price_provider,
                    system_program: system_program::ID,
                },
                &exchange::id(),
                &self.client.exchange_program_id(),
            ))
            .args(instruction::ExecuteOrder {
                recent_timestamp: self.recent_timestamp,
                execution_fee: self.execution_fee,
            })
            .accounts(
                feeds
                    .into_iter()
                    .chain(swap_markets)
                    .chain(swap_market_mints)
                    .collect::<Vec<_>>(),
            )
            .compute_budget(ComputeBudget::default().with_limit(EXECUTE_ORDER_COMPUTE_BUDGET));

        let mut pre_builder = self.client.exchange_request();
        let mut post_builder = self.client.exchange_request();

        // Merge claimable accounts.
        if let Some(account) = claimable_long_token_account_for_user.as_ref() {
            pre_builder = pre_builder.merge(self.client.use_claimable_account(
                &self.store,
                &hint.long_token_mint,
                &hint.user,
                self.recent_timestamp,
                account,
                0,
            ));
            post_builder = post_builder.merge(self.client.close_empty_claimable_account(
                &self.store,
                &hint.long_token_mint,
                &hint.user,
                self.recent_timestamp,
                account,
            ))
        }
        if let Some(account) = claimable_short_token_account_for_user.as_ref() {
            pre_builder = pre_builder.merge(self.client.use_claimable_account(
                &self.store,
                &hint.short_token_mint,
                &hint.user,
                self.recent_timestamp,
                account,
                0,
            ));
            post_builder = post_builder.merge(self.client.close_empty_claimable_account(
                &self.store,
                &hint.short_token_mint,
                &hint.user,
                self.recent_timestamp,
                account,
            ))
        }
        if let Some(account) = claimable_pnl_token_account_for_holding.as_ref() {
            let holding = hint.config.holding()?;
            pre_builder = pre_builder.merge(self.client.use_claimable_account(
                &self.store,
                &hint.pnl_token_mint,
                &holding,
                self.recent_timestamp,
                account,
                0,
            ));
            post_builder = post_builder.merge(self.client.close_empty_claimable_account(
                &self.store,
                &hint.pnl_token_mint,
                &holding,
                self.recent_timestamp,
                account,
            ))
        }
        let mut transaction_builder = TransactionBuilder::new(self.client.exchange().async_rpc());
        transaction_builder
            .try_push(pre_builder)?
            .try_push(execute_order)?
            .try_push(post_builder)?;
        Ok(transaction_builder)
    }
}
