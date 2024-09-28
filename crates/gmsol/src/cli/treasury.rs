use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::exchange::ExchangeOps;

use crate::{utils::Side, GMSOLClient};

#[derive(clap::Args)]
pub(super) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Claim fees.
    ClaimFees {
        market_token: Pubkey,
        #[arg(long)]
        side: Side,
    },
}

impl Args {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match self.command {
            Command::ClaimFees { market_token, side } => {
                let req = client
                    .claim_fees(store, &market_token, side.is_long())
                    .build()
                    .await?
                    .into_anchor_request_without_compute_budget();
                crate::utils::send_or_serialize(req, serialize_only, |signature| {
                    println!("{signature}");
                    Ok(())
                })
                .await
            }
        }
    }
}
