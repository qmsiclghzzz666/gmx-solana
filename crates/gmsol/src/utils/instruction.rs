use anchor_client::{anchor_lang::AnchorSerialize, solana_sdk::instruction::Instruction, Cluster};
use base64::{engine::general_purpose::STANDARD, Engine};
use solana_sdk::message::VersionedMessage;
use spl_governance::state::proposal_transaction::InstructionData;

/// Serialize an instruction.
pub fn serialize_instruction(ix: &Instruction) -> crate::Result<String> {
    let data = InstructionData::from(ix.clone()).try_to_vec()?;
    let encoded = STANDARD.encode(data);
    Ok(encoded)
}

/// Generate inspector url for the given message.
pub fn to_inspector_url(message: &VersionedMessage, cluster: Option<&Cluster>) -> String {
    use base64::{prelude::BASE64_STANDARD, Engine};
    use url::form_urlencoded;

    let message = BASE64_STANDARD.encode(message.serialize());

    let cluster = cluster.cloned().unwrap_or(Cluster::Mainnet);
    let encoded = form_urlencoded::Serializer::new(String::new())
        .append_pair("message", &message)
        .append_pair("cluster", &cluster.to_string())
        .finish();

    format!("https://explorer.solana.com/tx/inspector?{encoded}")
}
