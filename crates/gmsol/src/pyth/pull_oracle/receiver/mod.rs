use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_program},
    Program,
};
use pythnet_sdk::wire::v1::MerklePriceUpdate;

use crate::utils::RpcBuilder;

mod accounts;
mod instruction;

/// Treasury account seed.
pub const TREASURY_SEED: &[u8] = b"treasury";

/// Config account seed.
pub const CONFIG_SEED: &[u8] = b"config";

/// Find PDA for treasury account.
pub fn find_treasury_pda(treasury_id: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[TREASURY_SEED, &[treasury_id]],
        &pyth_solana_receiver_sdk::ID,
    )
}

/// Find PDA for config account.
pub fn find_config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG_SEED], &pyth_solana_receiver_sdk::ID)
}

/// Pyth Receiver Ops.
pub trait PythReceiverOps<C> {
    /// Post price update.
    fn post_price_update<'a>(
        &'a self,
        price_update: &'a Keypair,
        update: &MerklePriceUpdate,
        encoded_vaa: &Pubkey,
    ) -> RpcBuilder<'a, C, Pubkey>;
}

impl<S, C> PythReceiverOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn post_price_update<'a>(
        &'a self,
        price_update: &'a Keypair,
        update: &MerklePriceUpdate,
        encoded_vaa: &Pubkey,
    ) -> RpcBuilder<'a, C, Pubkey> {
        let treasury_id = rand::random();
        RpcBuilder::new(self)
            .with_output(price_update.pubkey())
            .args(instruction::PostUpdate {
                merkle_price_update: update.clone(),
                treasury_id,
            })
            .accounts(accounts::PostUpdate {
                payer: self.payer(),
                encoded_vaa: *encoded_vaa,
                config: find_config_pda().0,
                treasury: find_treasury_pda(treasury_id).0,
                price_update_account: price_update.pubkey(),
                system_program: system_program::ID,
                write_authority: self.payer(),
            })
            .signer(price_update)
    }
}
