use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::roles::{find_roles_address, RolesOps};

use crate::SharedClient;

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
    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match &self.action {
            Some(RolesAction::Get) | None => {
                let address = find_roles_address(store, &program.payer()).0;
                println!("{address}");
            }
            Some(RolesAction::Init { authority }) => {
                let authority = authority.unwrap_or(program.payer());
                let signature = program.initialize_roles(store, &authority).send().await?;
                tracing::info!("initialized a new roles account at {signature}");
            }
        }
        Ok(())
    }
}
