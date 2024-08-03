use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::exchange::ExchangeOps;

use crate::GMSOLClient;

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

#[derive(clap::ValueEnum, Clone, Copy)]
enum Side {
    Long,
    Short,
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
                    .claim_fees(store, &market_token, matches!(side, Side::Long))
                    .build()
                    .await?
                    .build_without_compute_budget();
                crate::utils::send_or_serialize(req, serialize_only, |signature| {
                    println!("{signature}");
                    Ok(())
                })
                .await
            }
        }
    }
}
