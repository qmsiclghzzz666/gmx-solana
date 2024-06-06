use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use data_store::{accounts, instruction};

/// Vault Operations.
pub trait VaultOps<C> {
    /// Initialize a market vault for the given token.
    fn initialize_market_vault(
        &self,
        store: &Pubkey,
        token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey);

    /// Transfer tokens out from the given market vault.
    fn market_vault_transfer_out(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> RequestBuilder<C>;
}

impl<C, S> VaultOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_market_vault(
        &self,
        store: &Pubkey,
        token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let vault = self.find_market_vault_address(store, token);
        let builder = self
            .data_store()
            .request()
            .accounts(accounts::InitializeMarketVault {
                authority,
                only_market_keeper: self.payer_roles_address(store),
                store: *store,
                mint: *token,
                vault,
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::InitializeMarketVault {
                market_token_mint: None,
            });
        (builder, vault)
    }

    fn market_vault_transfer_out(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::MarketVaultTransferOut {
                authority,
                only_controller: self.payer_roles_address(store),
                store: *store,
                market_vault: self.find_market_vault_address(store, token),
                to: *to,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::MarketVaultTransferOut { amount })
    }
}
