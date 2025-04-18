use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::utils::serde::StringPubkey;

use super::NonceBytes;

pub(crate) fn generate_nonce() -> NonceBytes {
    use rand::{distributions::Standard, Rng};

    let pubkey = rand::thread_rng()
        .sample_iter(Standard)
        .take(32)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    StringPubkey(pubkey)
}

pub(crate) fn prepare_ata(
    payer: &Pubkey,
    owner: &Pubkey,
    token: Option<&Pubkey>,
    token_program_id: &Pubkey,
) -> Option<(Pubkey, Instruction)> {
    use anchor_spl::associated_token::spl_associated_token_account::instruction;

    let token = token?;

    let ata = get_associated_token_address_with_program_id(owner, token, token_program_id);

    let prepare = instruction::create_associated_token_account_idempotent(
        payer,
        owner,
        token,
        token_program_id,
    );

    Some((ata, prepare))
}
