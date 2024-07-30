use std::sync::Arc;

use anchor_client::{solana_sdk::signature::Keypair, Cluster};
use futures_util::StreamExt;
use gmsol::{
    decode::{value::OwnedDataDecoder, Decode, GMSOLCPIEvent},
    pda::find_default_store,
    Client,
};
use solana_transaction_status::{UiInstruction, UiTransactionEncoding};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _store = std::env::var("STORE")
        .ok()
        .map(|store| store.parse())
        .transpose()?
        .unwrap_or(find_default_store().0);

    let client = Client::new(Cluster::Devnet, Arc::new(Keypair::new()))?;
    let event_authority = client.data_store_event_authority();
    let program_id = client.data_store_program_id();

    let mut stream = client
        .pub_sub()
        .await?
        .logs_subscribe(&event_authority, None)?;

    let query = client.data_store().async_rpc();
    while let Some(res) = stream.next().await {
        let Ok(res) = res else {
            continue;
        };
        let update = res.value();
        let signature = update.signature.parse()?;
        tracing::info!(%signature, "[0] received");
        let res = query
            .get_transaction(&signature, UiTransactionEncoding::Base58)
            .await?;
        let slot = res.slot;
        let Some(event_authority_idx) = res.transaction.transaction.decode().and_then(|tx| {
            tx.message
                .static_account_keys()
                .iter()
                .enumerate()
                .find_map(|(idx, pk)| (*pk == event_authority).then_some(idx))
        }) else {
            continue;
        };
        let Some(iixs) = res
            .transaction
            .meta
            .and_then(|meta| Option::<Vec<_>>::from(meta.inner_instructions))
        else {
            eyre::bail!("invalid encoding");
        };
        for ix in iixs.into_iter().flat_map(|ixs| ixs.instructions) {
            let UiInstruction::Compiled(ix) = ix else {
                continue;
            };
            if ix.accounts == [event_authority_idx as u8] {
                let data = ix.data;
                let data = bs58::decode(&data).into_vec()?;
                let decoder = OwnedDataDecoder::new(&program_id, &data);
                let event = GMSOLCPIEvent::decode(decoder)?;
                tracing::info!(%slot, "{event:?}");
            }
        }
    }
    tracing::info!("finished");
    Ok(())
}
