use std::ops::Deref;

use anchor_client::{
    anchor_lang::{prelude::AccountMeta, system_program},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use gmsol_store::{
    accounts, instruction,
    ops::glv::CreateGlvWithdrawalParams,
    states::{common::action::Action, glv::GlvWithdrawal, HasMarketMeta, NonceBytes},
};

use crate::{
    exchange::generate_nonce,
    store::token::TokenAccountOps,
    utils::{RpcBuilder, ZeroCopy},
};

/// Create GLV withdrawal builder.
pub struct CreateGlvWithdrawalBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    glv_token: Pubkey,
    market_token: Pubkey,
    final_long_token: Option<Pubkey>,
    final_short_token: Option<Pubkey>,
    long_token_swap_path: Vec<Pubkey>,
    short_token_swap_path: Vec<Pubkey>,
    glv_token_amount: u64,
    min_final_long_token_amount: u64,
    min_final_short_token_amount: u64,
    max_execution_lamports: u64,
    nonce: Option<NonceBytes>,
    glv_token_source: Option<Pubkey>,
    hint: Option<CreateGlvWithdrawalHint>,
}

/// Hint for [`CreateGlvWithdrawalBuilder`]
#[derive(Clone)]
pub struct CreateGlvWithdrawalHint {
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
}

impl CreateGlvWithdrawalHint {
    /// Create from market meta.
    pub fn new(meta: &impl HasMarketMeta) -> Self {
        let meta = meta.market_meta();
        Self {
            long_token_mint: meta.long_token_mint,
            short_token_mint: meta.short_token_mint,
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> CreateGlvWithdrawalBuilder<'a, C> {
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: Pubkey,
        glv_token: Pubkey,
        market_token: Pubkey,
        amount: u64,
    ) -> Self {
        Self {
            client,
            store,
            glv_token,
            market_token,
            final_long_token: None,
            final_short_token: None,
            long_token_swap_path: vec![],
            short_token_swap_path: vec![],
            glv_token_amount: amount,
            min_final_long_token_amount: 0,
            min_final_short_token_amount: 0,
            max_execution_lamports: GlvWithdrawal::MIN_EXECUTION_LAMPORTS,
            nonce: None,
            glv_token_source: None,
            hint: None,
        }
    }

    /// Set max execution fee allowed to use.
    pub fn max_execution_fee(&mut self, lamports: u64) -> &mut Self {
        self.max_execution_lamports = lamports;
        self
    }

    /// Set long swap path.
    pub fn long_token_swap_path(
        &mut self,
        final_long_token: &Pubkey,
        market_tokens: Vec<Pubkey>,
    ) -> &mut Self {
        self.final_long_token = Some(*final_long_token);
        self.long_token_swap_path = market_tokens;
        self
    }

    /// Set short swap path.
    pub fn short_token_swap_path(
        &mut self,
        final_short_token: &Pubkey,
        market_tokens: Vec<Pubkey>,
    ) -> &mut Self {
        self.final_short_token = Some(*final_short_token);
        self.short_token_swap_path = market_tokens;
        self
    }

    /// Set GLV token source.
    pub fn glv_token_source(&mut self, address: &Pubkey) -> &mut Self {
        self.glv_token_source = Some(*address);
        self
    }

    /// Set hint.
    pub fn hint(&mut self, hint: CreateGlvWithdrawalHint) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    fn market_address(&self) -> Pubkey {
        self.client
            .find_market_address(&self.store, &self.market_token)
    }

    async fn prepare_hint(&mut self) -> crate::Result<CreateGlvWithdrawalHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let market = self.market_address();
                let market = self.client.market(&market).await?;
                let hint = CreateGlvWithdrawalHint::new(&market);
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
        let glv_withdrawal = self
            .client
            .find_glv_withdrawal_address(&self.store, &owner, &nonce);
        let market = self.market_address();
        let glv = self.client.find_glv_address(&self.glv_token);
        let token_program_id = anchor_spl::token::ID;
        let glv_token_program_id = anchor_spl::token_2022::ID;

        let final_long_token = self.final_long_token.unwrap_or(hint.long_token_mint);
        let final_short_token = self.final_short_token.unwrap_or(hint.short_token_mint);

        let glv_token_source = self.glv_token_source.unwrap_or_else(|| {
            get_associated_token_address_with_program_id(
                &owner,
                &self.glv_token,
                &glv_token_program_id,
            )
        });

        let glv_token_escrow = get_associated_token_address_with_program_id(
            &glv_withdrawal,
            &self.glv_token,
            &glv_token_program_id,
        );
        let market_token_escrow = get_associated_token_address_with_program_id(
            &glv_withdrawal,
            &self.market_token,
            &token_program_id,
        );
        let final_long_token_escrow = get_associated_token_address_with_program_id(
            &glv_withdrawal,
            &final_long_token,
            &token_program_id,
        );
        let final_short_token_escrow = get_associated_token_address_with_program_id(
            &glv_withdrawal,
            &final_short_token,
            &token_program_id,
        );

        // Prepare the ATA for receiving final long tokens.
        let mut prepare = self.client.prepare_associated_token_account(
            &final_long_token,
            &token_program_id,
            None,
        );

        // Prepare the ATA for receiving final short tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &final_short_token,
            &token_program_id,
            None,
        ));

        // Prepare the escrow account for GLV tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &self.glv_token,
            &glv_token_program_id,
            Some(&glv_withdrawal),
        ));

        // Prepare the escrow account for market tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &self.market_token,
            &token_program_id,
            Some(&glv_withdrawal),
        ));

        // Prepare the escrow account for final long tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &final_long_token,
            &token_program_id,
            Some(&glv_withdrawal),
        ));
        // Prepare the escrow account for final long tokens.
        prepare = prepare.merge(self.client.prepare_associated_token_account(
            &final_short_token,
            &token_program_id,
            Some(&glv_withdrawal),
        ));

        let create = self
            .client
            .store_rpc()
            .accounts(accounts::CreateGlvWithdrawal {
                owner,
                store: self.store,
                market,
                glv,
                glv_withdrawal,
                glv_token: self.glv_token,
                market_token: self.market_token,
                final_long_token,
                final_short_token,
                glv_token_source,
                glv_token_escrow,
                market_token_escrow,
                final_long_token_escrow,
                final_short_token_escrow,
                system_program: system_program::ID,
                token_program: token_program_id,
                glv_token_program: glv_token_program_id,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::CreateGlvWithdrawal {
                nonce,
                params: CreateGlvWithdrawalParams {
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
                    glv_token_amount: self.glv_token_amount,
                    min_final_long_token_amount: self.min_final_long_token_amount,
                    min_final_short_token_amount: self.min_final_short_token_amount,
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

        Ok((prepare.merge(create), glv_withdrawal))
    }
}

/// Close GLV withdrawal builder.
pub struct CloseGlvWithdrawalBuilder<'a, C> {
    client: &'a crate::Client<C>,
    glv_withdrawal: Pubkey,
    reason: String,
    hint: Option<CloseGlvWithdrawalHint>,
}

/// Hint for [`CloseGlvDepositBuilder`].
#[derive(Clone)]
pub struct CloseGlvWithdrawalHint {
    store: Pubkey,
    owner: Pubkey,
    glv_token: Pubkey,
    market_token: Pubkey,
    final_long_token: Pubkey,
    final_short_token: Pubkey,
    market_token_escrow: Pubkey,
    final_long_token_escrow: Pubkey,
    final_short_token_escrow: Pubkey,
    glv_token_escrow: Pubkey,
}

impl CloseGlvWithdrawalHint {
    /// Create from the GLV withdrawal.
    pub fn new(glv_withdrawal: &GlvWithdrawal) -> Self {
        Self {
            store: *glv_withdrawal.header().store(),
            owner: *glv_withdrawal.header().owner(),
            glv_token: glv_withdrawal.tokens().glv_token(),
            market_token: glv_withdrawal.tokens().market_token(),
            final_long_token: glv_withdrawal.tokens().final_long_token(),
            final_short_token: glv_withdrawal.tokens().final_short_token(),
            market_token_escrow: glv_withdrawal.tokens().market_token_account(),
            final_long_token_escrow: glv_withdrawal.tokens().final_long_token_account(),
            final_short_token_escrow: glv_withdrawal.tokens().final_short_token_account(),
            glv_token_escrow: glv_withdrawal.tokens().glv_token_account(),
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> CloseGlvWithdrawalBuilder<'a, C> {
    pub(super) fn new(client: &'a crate::Client<C>, glv_withdrawal: Pubkey) -> Self {
        Self {
            client,
            glv_withdrawal,
            reason: "cancelled".to_string(),
            hint: None,
        }
    }

    /// Set hint.
    pub fn hint(&mut self, hint: CloseGlvWithdrawalHint) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    /// Set reason.
    pub fn reason(&mut self, reason: impl ToString) -> &mut Self {
        self.reason = reason.to_string();
        self
    }

    async fn prepare_hint(&mut self) -> crate::Result<CloseGlvWithdrawalHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let glv_deposit = self
                    .client
                    .account::<ZeroCopy<GlvWithdrawal>>(&self.glv_withdrawal)
                    .await?
                    .ok_or(crate::Error::NotFound)?
                    .0;
                let hint = CloseGlvWithdrawalHint::new(&glv_deposit);
                self.hint = Some(hint.clone());
                Ok(hint)
            }
        }
    }

    /// Build.
    pub async fn build(&mut self) -> crate::Result<RpcBuilder<'a, C>> {
        let hint = self.prepare_hint().await?;

        let token_program_id = anchor_spl::token::ID;
        let glv_token_program_id = anchor_spl::token_2022::ID;

        let payer = self.client.payer();

        let market_token_ata = get_associated_token_address_with_program_id(
            &hint.owner,
            &hint.market_token,
            &token_program_id,
        );
        let glv_token_ata = get_associated_token_address_with_program_id(
            &hint.owner,
            &hint.glv_token,
            &glv_token_program_id,
        );
        let final_long_token_ata = get_associated_token_address_with_program_id(
            &hint.owner,
            &hint.final_long_token,
            &token_program_id,
        );
        let final_short_token_ata = get_associated_token_address_with_program_id(
            &hint.owner,
            &hint.final_short_token,
            &token_program_id,
        );

        let rpc = self
            .client
            .store_rpc()
            .accounts(accounts::CloseGlvWithdrawal {
                executor: payer,
                store: hint.store,
                owner: hint.owner,
                glv_withdrawal: self.glv_withdrawal,
                market_token: hint.market_token,
                final_long_token: hint.final_long_token,
                final_short_token: hint.final_short_token,
                glv_token: hint.glv_token,
                market_token_escrow: hint.market_token_escrow,
                final_long_token_escrow: hint.final_long_token_escrow,
                final_short_token_escrow: hint.final_short_token_escrow,
                market_token_ata,
                final_long_token_ata,
                final_short_token_ata,
                glv_token_escrow: hint.glv_token_escrow,
                glv_token_ata,
                system_program: system_program::ID,
                token_program: token_program_id,
                glv_token_program: glv_token_program_id,
                associated_token_program: anchor_spl::associated_token::ID,
                event_authority: self.client.store_event_authority(),
                program: self.client.store_program_id(),
            })
            .args(instruction::CloseGlvWithdrawal {
                reason: self.reason.clone(),
            });
        Ok(rpc)
    }
}
