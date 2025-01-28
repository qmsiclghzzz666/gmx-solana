use anchor_client::anchor_lang;
use solana_sdk::{
    hash::Hash,
    instruction::CompiledInstruction,
    message::{
        v0::{Message, MessageAddressTableLookup},
        MessageHeader,
    },
};
use squads_multisig::squads_multisig_program::VaultTransaction;

/// Squads Vault Transaction.
pub struct SquadsVaultTransaction(VaultTransaction);

impl anchor_lang::Discriminator for SquadsVaultTransaction {
    const DISCRIMINATOR: [u8; 8] =
        <VaultTransaction as squads_multisig::anchor_lang::Discriminator>::DISCRIMINATOR;
}

impl anchor_lang::AccountDeserialize for SquadsVaultTransaction {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        let inner = <VaultTransaction as squads_multisig::anchor_lang::AccountDeserialize>::try_deserialize_unchecked(buf).map_err(|_err| {
            anchor_lang::error::ErrorCode::AccountDidNotDeserialize
        })?;

        Ok(Self(inner))
    }
}

impl SquadsVaultTransaction {
    /// Convert to transaction message.
    pub fn to_message(&self) -> Message {
        let message = &self.0.message;
        let instructions = message
            .instructions
            .iter()
            .map(|ix| CompiledInstruction {
                program_id_index: ix.program_id_index,
                accounts: ix.account_indexes.clone(),
                data: ix.data.clone(),
            })
            .collect();
        let address_table_lookups = message
            .address_table_lookups
            .iter()
            .map(|atl| MessageAddressTableLookup {
                account_key: atl.account_key,
                writable_indexes: atl.writable_indexes.clone(),
                readonly_indexes: atl.readonly_indexes.clone(),
            })
            .collect();
        let num_non_signers = message.account_keys.len() as u8 - message.num_signers;
        Message {
            header: MessageHeader {
                num_required_signatures: message.num_signers,
                num_readonly_signed_accounts: message.num_signers - message.num_writable_signers,
                num_readonly_unsigned_accounts: num_non_signers - message.num_writable_non_signers,
            },
            account_keys: message.account_keys.clone(),
            recent_blockhash: Hash::default(),
            instructions,
            address_table_lookups,
        }
    }
}
