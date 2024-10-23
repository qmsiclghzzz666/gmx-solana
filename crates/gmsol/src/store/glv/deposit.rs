use std::ops::Deref;

use anchor_client::{
    anchor_lang::{prelude::AccountMeta, system_program},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use gmsol_store::{
    accounts, instruction,
    ops::glv::CreateGlvDepositParams,
    states::{common::action::Action, GlvDeposit, HasMarketMeta, NonceBytes},
};

use crate::{exchange::generate_nonce, store::token::TokenAccountOps, utils::RpcBuilder};

/// Create GLV deposit builder.
pub struct CreateGlvDepositBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    glv_token: Pubkey,
    market_token: Pubkey,
    initial_long_token: Option<Pubkey>,
    initial_short_token: Option<Pubkey>,
    long_token_swap_path: Vec<Pubkey>,
    short_token_swap_path: Vec<Pubkey>,
    market_token_amount: u64,
    initial_long_token_amount: u64,
    initial_short_token_amount: u64,
    min_market_token_amount: u64,
    min_glv_token_amount: u64,
    max_execution_lamports: u64,
    nonce: Option<NonceBytes>,
    market_token_source: Option<Pubkey>,
    initial_long_token_source: Option<Pubkey>,
    initial_short_token_source: Option<Pubkey>,
    hint: Option<CreateGlvDepositHint>,
}

/// Hint for [`CreateGlvDepositBuilder`].
#[derive(Clone)]
pub struct CreateGlvDepositHint {
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
}

impl CreateGlvDepositHint {
    /// Create from market meta.
    pub fn new(meta: &impl HasMarketMeta) -> Self {
        let meta = meta.market_meta();
        Self {
            long_token_mint: meta.long_token_mint,
            short_token_mint: meta.short_token_mint,
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> CreateGlvDepositBuilder<'a, C> {
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: Pubkey,
        glv_token: Pubkey,
        market_token: Pubkey,
    ) -> Self {
        Self {
            client,
            store,
            glv_token,
            market_token,
            initial_long_token: None,
            initial_short_token: None,
            long_token_swap_path: vec![],
            short_token_swap_path: vec![],
            market_token_amount: 0,
            initial_long_token_amount: 0,
            initial_short_token_amount: 0,
            min_market_token_amount: 0,
            min_glv_token_amount: 0,
            max_execution_lamports: GlvDeposit::MIN_EXECUTION_LAMPORTS,
            nonce: None,
            market_token_source: None,
            initial_long_token_source: None,
            initial_short_token_source: None,
            hint: None,
        }
    }

    /// Set max execution fee allowed to use.
    pub fn max_execution_fee(&mut self, lamports: u64) -> &mut Self {
        self.max_execution_lamports = lamports;
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

    /// Set the market token amount and source to deposit with.
    pub fn market_token_deposit(
        &mut self,
        amount: u64,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.market_token_amount = amount;
        self.market_token_source = token_account.copied();
        self
    }

    /// Set the initial long token amount and source to deposit with.
    pub fn long_token_deposit(
        &mut self,
        amount: u64,
        token: Option<&Pubkey>,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.initial_long_token = token.copied();
        self.initial_long_token_amount = amount;
        self.initial_long_token_source = token_account.copied();
        self
    }

    /// Set the initial short tokens and source to deposit with.
    pub fn short_token_deposit(
        &mut self,
        amount: u64,
        token: Option<&Pubkey>,
        token_account: Option<&Pubkey>,
    ) -> &mut Self {
        self.initial_short_token = token.cloned();
        self.initial_short_token_amount = amount;
        self.initial_short_token_source = token_account.copied();
        self
    }

    /// Set hint.
    pub fn hint(&mut self, hint: CreateGlvDepositHint) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    fn market_address(&self) -> Pubkey {
        self.client
            .find_market_address(&self.store, &self.market_token)
    }

    async fn prepare_hint(&mut self) -> crate::Result<CreateGlvDepositHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let market = self.market_address();
                let market = self.client.market(&market).await?;
                let hint = CreateGlvDepositHint::new(&market);
                self.hint = Some(hint.clone());
                Ok(hint)
            }
        }
    }

    /// Build.
    pub async fn build_with_address(&mut self) -> crate::Result<(RpcBuilder<'a, C>, Pubkey)> {
        let hint = self.prepare_hint().await?;

        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let owner = self.client.payer();
        let glv_deposit = self
            .client
            .find_glv_deposit_address(&self.store, &owner, &nonce);
        let market = self.market_address();
        let glv = self.client.find_glv_address(&self.glv_token);
        let token_program_id = anchor_spl::token::ID;
        let glv_token_program_id = anchor_spl::token_2022::ID;

        let mut initial_long_token = None;
        let mut initial_short_token = None;

        let glv_token_escrow = get_associated_token_address_with_program_id(
            &glv_deposit,
            &self.glv_token,
            &glv_token_program_id,
        );
        let market_token_escrow = get_associated_token_address_with_program_id(
            &glv_deposit,
            &self.market_token,
            &token_program_id,
        );
        let mut initial_long_token_escrow = None;
        let mut initial_short_token_escrow = None;

        let mut market_token_source = None;
        let mut initial_long_token_source = None;
        let mut initial_short_token_source = None;

        // Prepare the ATA for receiving GLV tokens.
        let mut prepare = self.client.prepare_associated_token_account(
            &self.glv_token,
            &glv_token_program_id,
            None,
        );

        // Prepare the escrow account for GLV tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &self.glv_token,
            &glv_token_program_id,
            Some(&glv_deposit),
        ));

        // Prepare the escrow account for market tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &self.market_token,
            &token_program_id,
            Some(&glv_deposit),
        ));

        if self.market_token_amount != 0 {
            market_token_source = Some(self.market_token_source.unwrap_or_else(|| {
                get_associated_token_address_with_program_id(
                    &owner,
                    &self.market_token,
                    &token_program_id,
                )
            }));
        }

        if self.initial_long_token_amount != 0 {
            let token = self.initial_long_token.unwrap_or(hint.long_token_mint);
            initial_long_token = Some(token);
            initial_long_token_source = Some(self.initial_long_token_source.unwrap_or_else(|| {
                get_associated_token_address_with_program_id(&owner, &token, &token_program_id)
            }));
            initial_long_token_escrow = Some(get_associated_token_address_with_program_id(
                &glv_deposit,
                &token,
                &token_program_id,
            ));

            // Prepare the escrow account.
            prepare = prepare.merge(self.client.prepare_associated_token_account(
                &token,
                &token_program_id,
                Some(&glv_deposit),
            ));
        }

        if self.initial_short_token_amount != 0 {
            let token = self.initial_short_token.unwrap_or(hint.short_token_mint);
            initial_short_token = Some(token);
            initial_short_token_source =
                Some(self.initial_short_token_source.unwrap_or_else(|| {
                    get_associated_token_address_with_program_id(&owner, &token, &token_program_id)
                }));
            initial_short_token_escrow = Some(get_associated_token_address_with_program_id(
                &glv_deposit,
                &token,
                &token_program_id,
            ));

            // Prepare the escrow account.
            prepare = prepare.merge(self.client.prepare_associated_token_account(
                &token,
                &token_program_id,
                Some(&glv_deposit),
            ));
        }

        let create = self
            .client
            .store_rpc()
            .accounts(accounts::CreateGlvDeposit {
                owner,
                store: self.store,
                market,
                glv,
                glv_deposit,
                glv_token: self.glv_token,
                market_token: self.market_token,
                initial_long_token,
                initial_short_token,
                market_token_source,
                initial_long_token_source,
                initial_short_token_source,
                glv_token_escrow,
                market_token_escrow,
                initial_long_token_escrow,
                initial_short_token_escrow,
                system_program: system_program::ID,
                token_program: token_program_id,
                glv_token_program: glv_token_program_id,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::CreateGlvDeposit {
                nonce,
                params: CreateGlvDepositParams {
                    execution_lamports: self.max_execution_lamports,
                    long_token_swap_length: self
                        .long_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::invalid_argument("swap path too long"))?,
                    short_token_swap_length: self
                        .short_token_swap_path
                        .len()
                        .try_into()
                        .map_err(|_| crate::Error::invalid_argument("swap path too long"))?,
                    initial_long_token_amount: self.initial_long_token_amount,
                    initial_short_token_amount: self.initial_short_token_amount,
                    market_token_amount: self.market_token_amount,
                    min_market_token_amount: self.min_market_token_amount,
                    min_glv_token_amount: self.min_glv_token_amount,
                },
            })
            .accounts(
                self.long_token_swap_path
                    .iter()
                    .chain(self.short_token_swap_path.iter())
                    .map(|token| AccountMeta {
                        pubkey: self.client.find_market_address(&self.store, token),
                        is_signer: false,
                        is_writable: false,
                    })
                    .collect::<Vec<_>>(),
            );

        Ok((prepare.merge(create), glv_deposit))
    }
}
