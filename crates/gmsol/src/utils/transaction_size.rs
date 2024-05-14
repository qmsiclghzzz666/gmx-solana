use anchor_client::solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Estimated the size of the result transaction.
///
/// Based on the docs of [Solana Transactions](https://solana.com/docs/core/transactions),
/// and referring to the implementation of `@pythnetwork/solana-utils`.
pub fn transaction_size(ixs: &[Instruction], is_versioned_transaction: bool) -> usize {
    use std::collections::HashSet;

    fn get_size_of_compressed_u16(size: usize) -> usize {
        match size {
            0..=127 => 1,
            128..=16383 => 2,
            _ => 3,
        }
    }

    let mut programs = HashSet::<Pubkey>::default();
    let mut accounts = HashSet::<Pubkey>::default();
    let mut signers = HashSet::<Pubkey>::default();

    let ixs_size = ixs.iter().fold(0, |size, ix| {
        programs.insert(ix.program_id);
        accounts.insert(ix.program_id);
        ix.accounts.iter().for_each(|account| {
            accounts.insert(account.pubkey);
            if account.is_signer {
                signers.insert(account.pubkey);
            }
        });
        size + 1
            + get_size_of_compressed_u16(ix.accounts.len())
            + ix.accounts.len()
            + get_size_of_compressed_u16(ix.data.len())
            + ix.data.len()
    });

    get_size_of_compressed_u16(signers.len())
        + signers.len() * 64
        + 3
        + get_size_of_compressed_u16(accounts.len())
        + accounts.len() * 32
        + 32
        + get_size_of_compressed_u16(ixs.len())
        + ixs_size
        + (if is_versioned_transaction {
            1 + get_size_of_compressed_u16(0)
        } else {
            0
        })
}
