use std::{future::Future, ops::Deref};

use anchor_lang::system_program;
use gmsol_programs::gmsol_store::client::{accounts, args};
use gmsol_solana_utils::transaction_builder::TransactionBuilder;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

/// Operations for virtual inventory accounts.
pub trait VirtualInventoryOps<C> {
    /// Close a virtual inventory account.
    fn close_virtual_inventory_account(
        &self,
        store: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;

    /// Disable a virtual inventory.
    fn disable_virtual_inventory(
        &self,
        store: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;

    /// Leave a disabled virtual inventory.
    fn leave_disabled_virtual_inventory(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;

    /// Create a virtual inventory for swaps account.
    fn create_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        index: u32,
        long_amount_decimals: u8,
        short_amount_decimals: u8,
    ) -> crate::Result<TransactionBuilder<C, Pubkey>>;

    /// Join a virtual inventory for swaps.
    fn join_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_swaps: &Pubkey,
        hint_token_map: Option<&Pubkey>,
    ) -> impl Future<Output = crate::Result<TransactionBuilder<C>>>;

    /// Leave a virtual inventory for swaps.
    fn leave_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_swaps: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;

    /// Create a virtual inventory for positions account.
    fn create_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C, Pubkey>>;

    /// Join a virtual inventory for positions.
    fn join_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_positions: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;

    /// Leave a virtual inventory for positions.
    fn leave_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_positions: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>>;
}

impl<C: Deref<Target = impl Signer> + Clone> VirtualInventoryOps<C> for crate::Client<C> {
    fn close_virtual_inventory_account(
        &self,
        store: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::CloseVirtualInventory {
                authority: self.payer(),
                store: *store,
                store_wallet: self.find_store_wallet_address(store),
                virtual_inventory: *virtual_inventory,
            })
            .anchor_args(args::CloseVirtualInventory {});

        Ok(txn)
    }

    fn disable_virtual_inventory(
        &self,
        store: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::DisableVirtualInventory {
                authority: self.payer(),
                store: *store,
                virtual_inventory: *virtual_inventory,
            })
            .anchor_args(args::DisableVirtualInventory {});
        Ok(txn)
    }

    fn leave_disabled_virtual_inventory(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::LeaveDisabledVirtualInventory {
                authority: self.payer(),
                store: *store,
                virtual_inventory: *virtual_inventory,
                market: *market,
            })
            .anchor_args(args::LeaveDisabledVirtualInventory {});
        Ok(txn)
    }

    fn create_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        index: u32,
        long_amount_decimals: u8,
        short_amount_decimals: u8,
    ) -> crate::Result<TransactionBuilder<C, Pubkey>> {
        let virtual_inventory = self.find_virtual_inventory_for_swaps_address(store, index);
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::CreateVirtualInventoryForSwaps {
                authority: self.payer(),
                store: *store,
                virtual_inventory,
                system_program: system_program::ID,
            })
            .anchor_args(args::CreateVirtualInventoryForSwaps {
                index,
                long_amount_decimals,
                short_amount_decimals,
            })
            .output(virtual_inventory);
        Ok(txn)
    }

    async fn join_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_swaps: &Pubkey,
        hint_token_map: Option<&Pubkey>,
    ) -> crate::Result<TransactionBuilder<C>> {
        let token_map = match hint_token_map {
            Some(address) => *address,
            None => self
                .authorized_token_map_address(store)
                .await?
                .ok_or(crate::Error::NotFound)?,
        };
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::JoinVirtualInventoryForSwaps {
                authority: self.payer(),
                store: *store,
                token_map,
                virtual_inventory: *virtual_inventory_for_swaps,
                market: *market,
            })
            .anchor_args(args::JoinVirtualInventoryForSwaps {});

        Ok(txn)
    }

    fn leave_virtual_inventory_for_swaps(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_swaps: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::LeaveVirtualInventoryForSwaps {
                authority: self.payer(),
                store: *store,
                virtual_inventory: *virtual_inventory_for_swaps,
                market: *market,
            })
            .anchor_args(args::LeaveVirtualInventoryForSwaps {});

        Ok(txn)
    }

    fn create_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C, Pubkey>> {
        let virtual_inventory =
            self.find_virtual_inventory_for_positions_address(store, index_token);
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::CreateVirtualInventoryForPositions {
                authority: self.payer(),
                store: *store,
                index_token: *index_token,
                virtual_inventory,
                system_program: system_program::ID,
            })
            .anchor_args(args::CreateVirtualInventoryForPositions {})
            .output(virtual_inventory);
        Ok(txn)
    }

    fn join_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_positions: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::JoinVirtualInventoryForPositions {
                authority: self.payer(),
                store: *store,
                virtual_inventory: *virtual_inventory_for_positions,
                market: *market,
            })
            .anchor_args(args::JoinVirtualInventoryForPositions {});

        Ok(txn)
    }

    fn leave_virtual_inventory_for_positions(
        &self,
        store: &Pubkey,
        market: &Pubkey,
        virtual_inventory_for_positions: &Pubkey,
    ) -> crate::Result<TransactionBuilder<C>> {
        let txn = self
            .store_transaction()
            .anchor_accounts(accounts::LeaveVirtualInventoryForPositions {
                authority: self.payer(),
                store: *store,
                virtual_inventory: *virtual_inventory_for_positions,
                market: *market,
            })
            .anchor_args(args::LeaveVirtualInventoryForPositions {});

        Ok(txn)
    }
}
