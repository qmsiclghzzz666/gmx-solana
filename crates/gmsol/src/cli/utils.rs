use std::ops::Deref;

use anchor_client::{
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer},
    RequestBuilder,
};
use eyre::ContextCompat;
use gmsol::utils::TransactionBuilder;
use prettytable::format::{FormatBuilder, TableFormat};

#[derive(clap::Args, Clone)]
#[group(required = false, multiple = false, id = "oracle_address")]
pub(crate) struct Oracle {
    #[arg(long, env)]
    oracle: Option<Pubkey>,
    #[arg(long, default_value_t = 0)]
    oracle_index: u8,
}

impl Oracle {
    pub(crate) fn address(
        &self,
        store: Option<&Pubkey>,
        store_program_id: &Pubkey,
    ) -> gmsol::Result<Pubkey> {
        match self.oracle {
            Some(address) => Ok(address),
            None => Ok(gmsol::pda::find_oracle_address(
                store.wrap_err("`store` not provided")?,
                self.oracle_index,
                store_program_id,
            )
            .0),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Default)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum Output {
    /// Text.
    #[default]
    Text,
    /// Json.
    Json,
    /// Json Compact.
    JsonCompact,
}

impl Output {
    pub(crate) fn print<T: serde::Serialize>(
        &self,
        value: &T,
        text: impl FnOnce(&T) -> gmsol::Result<String>,
    ) -> gmsol::Result<()> {
        match self {
            Self::Text => {
                println!("{}", text(value)?);
            }
            Self::Json => {
                println!("{}", serde_json::to_string_pretty(value)?);
            }
            Self::JsonCompact => {
                println!("{}", serde_json::to_string(value)?);
            }
        }
        Ok(())
    }
}

pub(crate) fn generate_discriminator(name: &str) -> [u8; 8] {
    use anchor_syn::codegen::program::common::{sighash, SIGHASH_GLOBAL_NAMESPACE};
    use heck::AsSnakeCase;

    sighash(SIGHASH_GLOBAL_NAMESPACE, &AsSnakeCase(name).to_string())
}

pub(crate) async fn send_or_serialize<C, S>(
    req: RequestBuilder<'_, C>,
    serialize_only: bool,
    callback: impl FnOnce(Signature) -> gmsol::Result<()>,
) -> gmsol::Result<()>
where
    C: Clone + Deref<Target = S>,
    S: Signer,
{
    if serialize_only {
        for (idx, ix) in req.instructions()?.into_iter().enumerate() {
            println!("ix[{idx}]: {}", gmsol::utils::serialize_instruction(&ix)?);
        }
    } else {
        let signature = req.send().await?;
        (callback)(signature)?;
    }
    Ok(())
}

pub(crate) async fn send_or_serialize_transactions<C, S>(
    builder: TransactionBuilder<'_, C>,
    serialize_only: bool,
    skip_preflight: bool,
    callback: impl FnOnce(Vec<Signature>, Option<gmsol::Error>) -> gmsol::Result<()>,
) -> gmsol::Result<()>
where
    C: Clone + Deref<Target = S>,
    S: Signer,
{
    if serialize_only {
        for (idx, rpc) in builder.into_builders().into_iter().enumerate() {
            println!("Transaction {idx}:");
            for (idx, ix) in rpc
                .build_without_compute_budget()
                .instructions()?
                .into_iter()
                .enumerate()
            {
                println!("ix[{idx}]: {}", gmsol::utils::serialize_instruction(&ix)?);
            }
            println!();
        }
    } else {
        match builder
            .send_all_with_opts(
                None,
                RpcSendTransactionConfig {
                    skip_preflight,
                    ..Default::default()
                },
                false,
            )
            .await
        {
            Ok(signatures) => (callback)(signatures, None)?,
            Err((signatures, error)) => (callback)(signatures, Some(error))?,
        }
    }
    Ok(())
}

pub(crate) fn table_format() -> TableFormat {
    use prettytable::format::{LinePosition, LineSeparator};

    FormatBuilder::new()
        .padding(0, 2)
        .separator(LinePosition::Title, LineSeparator::new('-', '+', '+', '+'))
        .build()
}
