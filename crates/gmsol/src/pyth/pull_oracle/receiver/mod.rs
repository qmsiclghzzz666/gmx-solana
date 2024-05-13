use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_program},
    Program,
};
use pyth_sdk::Identifier;
use pythnet_sdk::wire::v1::MerklePriceUpdate;

use crate::{pyth::utils::parse_price_feed_message, utils::RpcBuilder};

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
    ) -> crate::Result<RpcBuilder<'a, C, (Identifier, Pubkey)>>;

    /// Reclaim rent.
    fn reclaim_rent(&self, price_update: &Pubkey) -> RpcBuilder<C>;
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
    ) -> crate::Result<RpcBuilder<'a, C, (Identifier, Pubkey)>> {
        let feed_id = parse_feed_id(update)?;
        let treasury_id = rand::random();
        Ok(RpcBuilder::new(self)
            .with_output((feed_id, price_update.pubkey()))
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
            .signer(price_update))
    }

    fn reclaim_rent(&self, price_update: &Pubkey) -> RpcBuilder<C> {
        RpcBuilder::new(self)
            .args(instruction::ReclaimRent {})
            .accounts(accounts::ReclaimRent {
                payer: self.payer(),
                price_update_account: *price_update,
            })
    }
}

fn parse_feed_id(update: &MerklePriceUpdate) -> crate::Result<Identifier> {
    let feed_id = parse_price_feed_message(update)?.feed_id;
    Ok(Identifier::new(feed_id))
}
