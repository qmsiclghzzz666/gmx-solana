use anchor_client::{anchor_lang::AnchorSerialize, solana_sdk::instruction::Instruction, Cluster};
use base64::{engine::general_purpose::STANDARD, Engine};
use solana_sdk::{message::VersionedMessage, pubkey::Pubkey};
use spl_governance::state::proposal_transaction::InstructionData;

/// Instruction serialziation format.
#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum InstructionSerialization {
    /// SPL-Governance format.
    #[default]
    Gov,
    /// Base58 (Squads).
    Base58,
    /// Base64.
    Base64,
}

/// Serialize an instruction.
pub fn serialize_instruction(
    ix: &Instruction,
    format: InstructionSerialization,
    payer: Option<&Pubkey>,
) -> crate::Result<String> {
    use solana_sdk::message::legacy::Message;

    let message = match format {
        InstructionSerialization::Gov => {
            let data = InstructionData::from(ix.clone()).try_to_vec()?;
            STANDARD.encode(data)
        }
        InstructionSerialization::Base58 | InstructionSerialization::Base64 => {
            let message = Message::new(&[ix.clone()], payer);
            match format {
                InstructionSerialization::Base58 => bs58::encode(message.serialize()).into_string(),
                InstructionSerialization::Base64 => STANDARD.encode(message.serialize()),
                _ => {
                    unreachable!()
                }
            }
        }
    };

    Ok(message)
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
