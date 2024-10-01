use std::{future::Future, ops::Deref};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_store::{
    accounts, instruction,
    states::user::{ReferralCode, ReferralCodeBytes},
};

use crate::utils::{RpcBuilder, ZeroCopy};

/// User Account Operations.
pub trait UserOps<C> {
    /// Prepare User.
    fn prepare_user(&self, store: &Pubkey) -> crate::Result<RpcBuilder<C>>;

    /// Initialize Referral Code.
    fn initialize_referral_code(
        &self,
        store: &Pubkey,
        code: ReferralCodeBytes,
    ) -> crate::Result<RpcBuilder<C>>;

    /// Set referrer.
    fn set_referrer(
        &self,
        store: &Pubkey,
        code: [u8; 4],
        hint_referrer: Option<Pubkey>,
    ) -> impl Future<Output = crate::Result<RpcBuilder<C>>>;
}

impl<C: Deref<Target = impl Signer> + Clone> UserOps<C> for crate::Client<C> {
    fn prepare_user(&self, store: &Pubkey) -> crate::Result<RpcBuilder<C>> {
        let owner = self.payer();
        let user = self.find_user_address(store, &owner);
        let rpc = self
            .data_store_rpc()
            .accounts(accounts::PrepareUser {
                owner,
                store: *store,
                user,
                system_program: system_program::ID,
            })
            .args(instruction::PrepareUser {});
        Ok(rpc)
    }

    fn initialize_referral_code(
        &self,
        store: &Pubkey,
        code: ReferralCodeBytes,
    ) -> crate::Result<RpcBuilder<C>> {
        let owner = self.payer();
        let referral_code = self.find_referral_code_address(store, code);
        let user = self.find_user_address(store, &owner);
        let rpc = self
            .data_store_rpc()
            .accounts(accounts::InitializeReferralCode {
                owner,
                store: *store,
                referral_code,
                user,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeReferralCode { code });
        Ok(rpc)
    }

    async fn set_referrer(
        &self,
        store: &Pubkey,
        code: [u8; 4],
        hint_referrer: Option<Pubkey>,
    ) -> crate::Result<RpcBuilder<C>> {
        let owner = self.payer();
        let user = self.find_user_address(store, &owner);

        let referral_code = self.find_referral_code_address(store, code);

        let referrer = match hint_referrer {
            Some(referrer) => referrer,
            None => {
                let code = self
                    .account::<ZeroCopy<ReferralCode>>(&referral_code)
                    .await?
                    .ok_or(crate::Error::NotFound)?
                    .0;
                let owner = code.owner;
                self.find_user_address(store, &owner)
            }
        };

        let rpc = self
            .data_store_rpc()
            .accounts(accounts::SetReferrer {
                owner,
                store: *store,
                user,
                referral_code,
                referrer,
            })
            .args(instruction::SetReferrer { code });

        Ok(rpc)
    }
}
