use std::{ops::Deref, sync::Arc};

use anchor_client::{
    anchor_lang::{system_program, Id},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address;
use gmsol_exchange::{accounts, instruction};
use gmsol_store::states::{
    common::TokensWithFeed, MarketMeta, NonceBytes, Position, Pyth, Store, TokenMap,
};

use crate::{
    store::utils::FeedsParser,
    utils::{ComputeBudget, TransactionBuilder},
};

use super::{
    generate_nonce,
    order::{recent_timestamp, ClaimableAccountsBuilder},
};

/// The compute budget for `liquidate`.
pub const LIQUIDATE_COMPUTE_BUDGET: u32 = 800_000;

#[cfg(feature = "pyth-pull-oracle")]
use crate::pyth::pull_oracle::{ExecuteWithPythPrices, Prices, PythPullOracleContext};

/// Liquidate Instruction Builder.
pub struct LiquidateBuilder<'a, C> {
    client: &'a crate::Client<C>,
    nonce: Option<NonceBytes>,
    recent_timestamp: i64,
    execution_fee: u64,
    oracle: Pubkey,
    position: Pubkey,
    price_provider: Pubkey,
    hint: Option<LiquidateHint>,
    feeds_parser: FeedsParser,
}

/// Hint for liquidation.
#[derive(Clone)]
pub struct LiquidateHint {
    store_address: Pubkey,
    owner: Pubkey,
    token_map: Pubkey,
    market: Pubkey,
    pnl_token: Pubkey,
    store: Arc<Store>,
    meta: MarketMeta,
    tokens_with_feed: TokensWithFeed,
}

impl LiquidateHint {
    /// Create from position.
    pub async fn from_position<C: Deref<Target = impl Signer> + Clone>(
        client: &crate::Client<C>,
        position: &Position,
    ) -> crate::Result<Self> {
        let store_address = position.store;
        let store = client.store(&store_address).await?;
        let token_map_address = client
            .authorized_token_map_address(&store_address)
            .await?
            .ok_or(crate::Error::invalid_argument(
                "token map is not configurated for the store",
            ))?;
        let token_map = client.token_map(&token_map_address).await?;
        let market = client.find_market_address(&store_address, &position.market_token);
        let meta = *client.market(&market).await?.meta();

        Self::try_new(position, Arc::new(store), &token_map, market, meta)
    }

    /// Create a new hint.
    pub fn try_new(
        position: &Position,
        store: Arc<Store>,
        token_map: &TokenMap,
        market: Pubkey,
        market_meta: MarketMeta,
    ) -> crate::Result<Self> {
        use gmsol_exchange::utils::token_records;

        let records = token_records(
            token_map,
            &[
                market_meta.index_token_mint,
                market_meta.long_token_mint,
                market_meta.short_token_mint,
            ]
            .into(),
        )?;
        let tokens_with_feed = TokensWithFeed::try_from_records(records)?;

        Ok(Self {
            store_address: position.store,
            owner: position.owner,
            token_map: *store.token_map().ok_or(crate::Error::invalid_argument(
                "missing token map for the store",
            ))?,
            market,
            store,
            tokens_with_feed,
            pnl_token: market_meta.pnl_token(position.try_is_long()?),
            meta: market_meta,
        })
    }

    /// Get feeds.
    pub fn feeds(&self) -> &TokensWithFeed {
        &self.tokens_with_feed
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> LiquidateBuilder<'a, C> {
    pub(super) fn try_new(
        client: &'a crate::Client<C>,
        oracle: &Pubkey,
        position: &Pubkey,
    ) -> crate::Result<Self> {
        Ok(Self {
            client,
            oracle: *oracle,
            recent_timestamp: recent_timestamp()?,
            position: *position,
            price_provider: Pyth::id(),
            execution_fee: 0,
            nonce: None,
            hint: None,
            feeds_parser: Default::default(),
        })
    }

    /// Prepare hint for liquidation.
    pub async fn prepare_hint(&mut self) -> crate::Result<LiquidateHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let position = self.client.position(&self.position).await?;
                let hint = LiquidateHint::from_position(self.client, &position).await?;
                self.hint = Some(hint.clone());
                Ok(hint)
            }
        }
    }

    /// Set hint with the given position for the liquidation.
    pub fn hint(&mut self, hint: LiquidateHint) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    /// Set price provider to the given.
    pub fn price_provider(&mut self, program: &Pubkey) -> &mut Self {
        self.price_provider = *program;
        self
    }

    /// Set execution fee.
    pub fn execution_fee(&mut self, lamports: u64) -> &mut Self {
        self.execution_fee = lamports;
        self
    }

    /// Parse feeds with the given price udpates map.
    #[cfg(feature = "pyth-pull-oracle")]
    pub fn parse_with_pyth_price_updates(&mut self, price_updates: Prices) -> &mut Self {
        self.feeds_parser.with_pyth_price_updates(price_updates);
        self
    }

    /// Build [`TransactionBuilder`] for liquidating the position.
    pub async fn build(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let hint = self.prepare_hint().await?;
        let owner = hint.owner;
        let store = hint.store_address;
        let meta = &hint.meta;
        let long_token_mint = meta.long_token_mint;
        let short_token_mint = meta.short_token_mint;

        let time_key = hint.store.claimable_time_key(self.recent_timestamp)?;
        let claimable_long_token_account_for_user =
            self.client
                .find_claimable_account_address(&store, &long_token_mint, &owner, &time_key);
        let claimable_short_token_account_for_user = self.client.find_claimable_account_address(
            &store,
            &short_token_mint,
            &owner,
            &time_key,
        );
        let claimable_pnl_token_account_for_holding = self.client.find_claimable_account_address(
            &store,
            &hint.pnl_token,
            hint.store.holding(),
            &time_key,
        );
        let feeds = self
            .feeds_parser
            .parse(hint.feeds())
            .collect::<Result<Vec<_>, _>>()?;

        let exec_builder = self
            .client
            .exchange_rpc()
            .accounts(accounts::Liquidate {
                authority: self.client.payer(),
                owner,
                controller: self.client.controller_address(&store),
                store,
                token_map: hint.token_map,
                oracle: self.oracle,
                market: hint.market,
                market_token_mint: meta.market_token_mint,
                long_token_mint,
                short_token_mint,
                order: self.client.find_order_address(&store, &owner, &nonce),
                position: self.position,
                long_token_vault: self
                    .client
                    .find_market_vault_address(&store, &long_token_mint),
                short_token_vault: self
                    .client
                    .find_market_vault_address(&store, &short_token_mint),
                long_token_account: get_associated_token_address(&owner, &long_token_mint),
                short_token_account: get_associated_token_address(&owner, &short_token_mint),
                claimable_long_token_account_for_user,
                claimable_short_token_account_for_user,
                claimable_pnl_token_account_for_holding,
                event_authority: self.client.data_store_event_authority(),
                data_store_program: self.client.store_program_id(),
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
                price_provider: self.price_provider,
                system_program: system_program::ID,
            })
            .args(instruction::Liquidate {
                recent_timestamp: self.recent_timestamp,
                nonce,
                execution_fee: self.execution_fee,
            })
            .accounts(feeds)
            .compute_budget(ComputeBudget::default().with_limit(LIQUIDATE_COMPUTE_BUDGET));

        let (pre_builder, post_builder) = ClaimableAccountsBuilder::new(
            self.recent_timestamp,
            store,
            owner,
            *hint.store.holding(),
        )
        .claimable_long_token_account_for_user(
            &long_token_mint,
            &claimable_long_token_account_for_user,
        )
        .claimable_short_token_account_for_user(
            &short_token_mint,
            &claimable_short_token_account_for_user,
        )
        .claimable_pnl_token_account_for_holding(
            &hint.pnl_token,
            &claimable_pnl_token_account_for_holding,
        )
        .build(self.client);

        let mut builder = TransactionBuilder::new(self.client.exchange().async_rpc());
        builder
            .try_push(pre_builder)?
            .try_push(exec_builder)?
            .try_push(post_builder)?;
        Ok(builder)
    }
}

#[cfg(feature = "pyth-pull-oracle")]
impl<'a, C: Deref<Target = impl Signer> + Clone> ExecuteWithPythPrices<'a, C>
    for LiquidateBuilder<'a, C>
{
    fn set_execution_fee(&mut self, lamports: u64) {
        self.execution_fee(lamports);
    }

    async fn context(&mut self) -> crate::Result<PythPullOracleContext> {
        let hint = self.prepare_hint().await?;
        let ctx = PythPullOracleContext::try_from_feeds(hint.feeds())?;
        Ok(ctx)
    }

    async fn build_rpc_with_price_updates(
        &mut self,
        price_updates: Prices,
    ) -> crate::Result<Vec<crate::utils::RpcBuilder<'a, C, ()>>> {
        let tx = self
            .parse_with_pyth_price_updates(price_updates)
            .build()
            .await?;
        Ok(tx.into_builders())
    }
}
