use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{accounts, constants, instruction};

use super::roles::find_roles_address;

/// Find PDA for the market vault.
pub fn find_market_vault_address(store: &Pubkey, token: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            constants::MARKET_VAULT_SEED,
            store.as_ref(),
            token.as_ref(),
            &[],
        ],
        &data_store::id(),
    )
}

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

impl<C, S> VaultOps<C> for Program<C>
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
        let vault = find_market_vault_address(store, token).0;
        let builder = self
            .request()
            .accounts(accounts::InitializeMarketVault {
                authority,
                only_market_keeper: find_roles_address(store, &authority).0,
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
        self.request()
            .accounts(accounts::MarketVaultTransferOut {
                authority,
                only_controller: find_roles_address(store, &authority).0,
                store: *store,
                market_vault: find_market_vault_address(store, token).0,
                to: *to,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::MarketVaultTransferOut { amount })
    }
}
