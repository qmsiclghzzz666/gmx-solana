use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use anchor_spl::associated_token::get_associated_token_address;
use data_store::states::{withdrawal::TokenParams, Market, NonceBytes, Seed, Withdrawal};
use exchange::{
    accounts, instruction, instructions::CreateWithdrawalParams, utils::ControllerSeeds,
};

use crate::store::{
    data_store::{find_market_address, find_market_vault_address, find_token_config_map},
    roles::find_roles_address,
};

use super::generate_nonce;

/// Create PDA for withdrawal.
pub fn find_withdrawal_address(store: &Pubkey, user: &Pubkey, nonce: &NonceBytes) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Withdrawal::SEED, store.as_ref(), user.as_ref(), nonce],
        &data_store::id(),
    )
}

/// Create Withdrawal Builder.
pub struct CreateWithdrawalBuilder<'a, C> {
    program: &'a Program<C>,
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
}

impl<'a, C, S> CreateWithdrawalBuilder<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    pub(super) fn new(
        program: &'a Program<C>,
        store: Pubkey,
        market_token: Pubkey,
        amount: u64,
    ) -> Self {
        Self {
            program,
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
        self.final_long_token = Some(*token);
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
            None => get_associated_token_address(&self.program.payer(), &self.market_token),
        }
    }

    fn get_or_find_associated_final_long_token_account(&self, token: &Pubkey) -> Pubkey {
        match self.final_long_token_receiver {
            Some(account) => account,
            None => get_associated_token_address(&self.program.payer(), token),
        }
    }

    fn get_or_find_associated_final_short_token_account(&self, token: &Pubkey) -> Pubkey {
        match self.final_short_token_receiver {
            Some(account) => account,
            None => get_associated_token_address(&self.program.payer(), token),
        }
    }

    async fn get_or_fetch_final_tokens(&self, market: &Pubkey) -> crate::Result<(Pubkey, Pubkey)> {
        if let (Some(long_token), Some(short_token)) =
            (self.final_long_token, self.final_short_token)
        {
            return Ok((long_token, short_token));
        }
        let market: Market = self.program.account(*market).await?;
        Ok((
            self.final_long_token
                .unwrap_or_else(|| market.meta().long_token_mint),
            self.final_short_token
                .unwrap_or_else(|| market.meta().short_token_mint),
        ))
    }

    /// Create the [`RequestBuilder`] and return withdrawal address.
    pub async fn build_with_address(&self) -> crate::Result<(RequestBuilder<'a, C>, Pubkey)> {
        let payer = self.program.payer();
        let authority = ControllerSeeds::find_with_address(&self.store).1;
        let market_token_withdrawal_vault =
            find_market_vault_address(&self.store, &self.market_token).0;
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let withdrawal = find_withdrawal_address(&self.store, &payer, &nonce).0;
        let market = find_market_address(&self.store, &self.market_token).0;
        let (long_token, short_token) = self.get_or_fetch_final_tokens(&market).await?;
        let builder = self
            .program
            .request()
            .accounts(accounts::CreateWithdrawal {
                authority,
                store: self.store,
                only_controller: find_roles_address(&self.store, &authority).0,
                data_store_program: data_store::id(),
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                token_config_map: find_token_config_map(&self.store).0,
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
                        pubkey: find_market_address(&self.store, mint).0,
                        is_signer: false,
                        is_writable: false,
                    })
                    .collect::<Vec<_>>(),
            );

        Ok((builder, withdrawal))
    }
}
