use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address;
use gmsol_store::{
    accounts, instruction,
    ops::shift::CreateShiftParams,
    states::{common::action::Action, NonceBytes, Shift},
};

use crate::{
    exchange::generate_nonce,
    utils::{RpcBuilder, ZeroCopy},
};

/// Create Shift Builder.
pub struct CreateShiftBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    from_market_token: Pubkey,
    to_market_token: Pubkey,
    execution_fee: u64,
    amount: u64,
    min_to_market_token_amount: u64,
    nonce: Option<NonceBytes>,
    hint: CreateShiftHint,
}

/// Hint for creating shift.
#[derive(Default)]
pub struct CreateShiftHint {
    from_market_token_source: Option<Pubkey>,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> CreateShiftBuilder<'a, C> {
    pub(super) fn new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        from_market_token: &Pubkey,
        to_market_token: &Pubkey,
        amount: u64,
    ) -> Self {
        Self {
            client,
            store: *store,
            from_market_token: *from_market_token,
            to_market_token: *to_market_token,
            execution_fee: Shift::MIN_EXECUTION_LAMPORTS,
            amount,
            min_to_market_token_amount: 0,
            nonce: None,
            hint: Default::default(),
        }
    }

    /// Set exectuion fee allowed to use.
    pub fn execution_fee(&mut self, amount: u64) -> &mut Self {
        self.execution_fee = amount;
        self
    }

    /// Set min to market token amount.
    pub fn min_to_market_token_amount(&mut self, amount: u64) -> &mut Self {
        self.min_to_market_token_amount = amount;
        self
    }

    /// Set hint.
    pub fn hint(&mut self, hint: CreateShiftHint) -> &mut Self {
        self.hint = hint;
        self
    }

    fn get_from_market_token_source(&self) -> Pubkey {
        match self.hint.from_market_token_source {
            Some(address) => address,
            None => get_associated_token_address(&self.client.payer(), &self.from_market_token),
        }
    }

    fn get_create_shift_params(&self) -> CreateShiftParams {
        CreateShiftParams {
            execution_lamports: self.execution_fee,
            from_market_token_amount: self.amount,
            min_to_market_token_amount: self.min_to_market_token_amount,
        }
    }

    /// Build a [`RpcBuilder`] to create shift account and return the address of the shift account to create.
    pub fn build_with_address(&self) -> crate::Result<(RpcBuilder<'a, C>, Pubkey)> {
        let owner = self.client.payer();
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let shift = self.client.find_shift_address(&self.store, &owner, &nonce);

        let from_market = self
            .client
            .find_market_address(&self.store, &self.from_market_token);
        let to_market = self
            .client
            .find_market_address(&self.store, &self.to_market_token);

        let from_market_token_escrow =
            get_associated_token_address(&shift, &self.from_market_token);
        let to_market_token_escrow = get_associated_token_address(&shift, &self.to_market_token);
        let to_market_token_ata = get_associated_token_address(&owner, &self.to_market_token);

        let prepare = self
            .client
            .store_rpc()
            .accounts(accounts::PrepareShiftEscorw {
                owner,
                store: self.store,
                shift,
                from_market_token: self.from_market_token,
                to_market_token: self.to_market_token,
                from_market_token_escrow,
                to_market_token_escrow,
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::PrepareShiftEscrow { nonce });

        let rpc = self
            .client
            .store_rpc()
            .accounts(accounts::CreateShift {
                owner,
                store: self.store,
                from_market,
                to_market,
                shift,
                from_market_token: self.from_market_token,
                to_market_token: self.to_market_token,
                from_market_token_escrow,
                to_market_token_escrow,
                from_market_token_source: self.get_from_market_token_source(),
                to_market_token_ata,
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::CreateShift {
                nonce,
                params: self.get_create_shift_params(),
            });

        Ok((prepare.merge(rpc), shift))
    }
}

/// Close Shift Builder.
pub struct CloseShiftBuilder<'a, C> {
    client: &'a crate::Client<C>,
    shift: Pubkey,
    reason: String,
    hint: Option<CloseShiftHint>,
}

/// Hint for `close_shift` instruction.
#[derive(Clone)]
pub struct CloseShiftHint {
    store: Pubkey,
    owner: Pubkey,
    from_market_token: Pubkey,
    to_market_token: Pubkey,
    from_market_token_escrow: Pubkey,
    to_market_token_escrow: Pubkey,
}

impl CloseShiftHint {
    /// Create hint for `close_shift` instruction.
    pub fn new(shift: &Shift) -> crate::Result<Self> {
        let tokens = shift.tokens();
        Ok(Self {
            store: *shift.header().store(),
            owner: *shift.header().owner(),
            from_market_token: tokens.from_market_token(),
            from_market_token_escrow: tokens.from_market_token_account(),
            to_market_token: tokens.to_market_token(),
            to_market_token_escrow: tokens.to_market_token_account(),
        })
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_market_token_ata(&self) -> Pubkey {
        get_associated_token_address(&self.owner, &self.from_market_token)
    }

    fn to_market_token_ata(&self) -> Pubkey {
        get_associated_token_address(&self.owner, &self.to_market_token)
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> CloseShiftBuilder<'a, C> {
    pub(super) fn new(client: &'a crate::Client<C>, shift: &Pubkey) -> Self {
        Self {
            client,
            shift: *shift,
            hint: None,
            reason: String::from("cancelled"),
        }
    }

    /// Set hint.
    pub fn hint(&mut self, hint: CloseShiftHint) -> &mut Self {
        self.hint = Some(hint);
        self
    }

    /// Set reason.
    pub fn reason(&mut self, reason: impl ToString) -> &mut Self {
        self.reason = reason.to_string();
        self
    }

    /// Prepare hint if needed
    pub async fn prepare_hint(&mut self) -> crate::Result<CloseShiftHint> {
        let hint = match &self.hint {
            Some(hint) => hint.clone(),
            None => {
                let shift = self
                    .client
                    .account::<ZeroCopy<_>>(&self.shift)
                    .await?
                    .ok_or(crate::Error::NotFound)?;
                let hint = CloseShiftHint::new(&shift.0)?;
                self.hint = Some(hint.clone());
                hint
            }
        };

        Ok(hint)
    }

    /// Build a [`RpcBuilder`] to close shift account.
    pub async fn build(&mut self) -> crate::Result<RpcBuilder<'a, C>> {
        let hint = self.prepare_hint().await?;
        let executor = self.client.payer();
        let rpc = self
            .client
            .store_rpc()
            .accounts(accounts::CloseShift {
                executor,
                store: hint.store,
                owner: hint.owner,
                shift: self.shift,
                from_market_token: hint.from_market_token,
                to_market_token: hint.to_market_token,
                from_market_token_escrow: hint.from_market_token_escrow,
                to_market_token_escrow: hint.to_market_token_escrow,
                from_market_token_ata: hint.from_market_token_ata(),
                to_market_token_ata: hint.to_market_token_ata(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
                event_authority: self.client.store_event_authority(),
                program: self.client.store_program_id(),
            })
            .args(instruction::CloseShift {
                reason: self.reason.clone(),
            });

        Ok(rpc)
    }
}
