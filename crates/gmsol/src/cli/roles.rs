use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::roles::RolesOps;

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct RolesArgs {
    #[command(subcommand)]
    action: Option<RolesAction>,
}

#[derive(clap::Subcommand)]
enum RolesAction {
    /// Get.
    Get,
    /// Init.
    Init {
        /// Authority.
        #[arg(long)]
        authority: Option<Pubkey>,
    },
}

impl RolesArgs {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.action {
            Some(RolesAction::Get) | None => {
                let address = client.payer_roles_address(store);
                println!("{address}");
            }
            Some(RolesAction::Init { authority }) => {
                crate::utils::send_or_serialize(
                    client.initialize_roles(store, &authority.unwrap_or(client.payer())),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized a new roles account at {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}
