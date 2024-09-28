use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::market::MarketOps;
use gmsol_model::PoolKind;

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Turn an impure pool into a pure pool.
    TurnIntoPurePool {
        market_token: Pubkey,
        kind: PoolKind,
    },
    /// Turn a pure pool into a impure pool.
    TurnIntoImpurePool {
        market_token: Pubkey,
        kind: PoolKind,
    },
}

impl Args {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.command {
            Command::TurnIntoPurePool { market_token, kind } => {
                crate::utils::send_or_serialize(
                    client
                        .turn_into_pure_pool(store, market_token, *kind)
                        .into_anchor_request_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("turned into pure pool at {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::TurnIntoImpurePool { market_token, kind } => {
                crate::utils::send_or_serialize(
                    client
                        .turn_into_impure_pool(store, market_token, *kind)
                        .into_anchor_request_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("turned into impure pool at {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}
