use std::{future::Future, ops::Deref};

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction},
    ClientError, Program, RequestBuilder,
};

mod accounts;
mod instruction;

/// Wormhole Core Bridge Program Address.
pub const WORMHOLE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    241, 11, 180, 229, 13, 86, 253, 161, 61, 254, 31, 50, 155, 141, 57, 61, 210, 74, 1, 69, 145,
    225, 131, 22, 151, 148, 13, 124, 52, 163, 141, 221,
]);

/// The start offset of the VAA bytes.
pub const VAA_START: u64 = 46;

/// Wormhole Ops.
pub trait WormholeOps<C> {
    /// Create and initialize encoded vaa account.
    fn create_encoded_vaa<'a>(
        &'a self,
        encoded_vaa: &'a Keypair,
        vaa_buffer_len: u64,
    ) -> impl Future<Output = crate::Result<RequestBuilder<'a, C>>>;

    /// Write to encoded vaa account.
    fn write_encoded_vaa(&self, draft_vaa: &Pubkey, index: u32, data: &[u8]) -> RequestBuilder<C>;
}

impl<S, C> WormholeOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    async fn create_encoded_vaa<'a>(
        &'a self,
        encoded_vaa: &'a Keypair,
        vaa_buffer_len: u64,
    ) -> crate::Result<RequestBuilder<'a, C>> {
        let space = vaa_buffer_len + VAA_START;
        let lamports = self
            .async_rpc()
            .get_minimum_balance_for_rent_exemption(space as usize)
            .await
            .map_err(ClientError::from)?;
        let request = self
            .request()
            .instruction(system_instruction::create_account(
                &self.payer(),
                &encoded_vaa.pubkey(),
                lamports,
                space,
                &self.id(),
            ))
            .args(instruction::InitEncodedVaa {})
            .accounts(accounts::InitEncodedVaa {
                write_authority: self.payer(),
                encoded_vaa: encoded_vaa.pubkey(),
            })
            .signer(encoded_vaa);
        Ok(request)
    }

    fn write_encoded_vaa(&self, draft_vaa: &Pubkey, index: u32, data: &[u8]) -> RequestBuilder<C> {
        self.request()
            .args(instruction::WriteEncodedVaa {
                index,
                data: data.to_owned(),
            })
            .accounts(accounts::WriteEncodedVaa {
                write_authority: self.payer(),
                draft_vaa: *draft_vaa,
            })
    }
}
