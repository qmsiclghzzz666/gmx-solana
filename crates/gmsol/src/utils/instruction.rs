use anchor_client::{anchor_lang::AnchorSerialize, solana_sdk::instruction::Instruction};
use base64::{engine::general_purpose::STANDARD, Engine};
use spl_governance::state::proposal_transaction::InstructionData;

/// Serialize an instruction.
pub fn serialize_instruction(ix: &Instruction) -> crate::Result<String> {
    let data = InstructionData::from(ix.clone()).try_to_vec()?;
    let encoded = STANDARD.encode(data);
    Ok(encoded)
}
