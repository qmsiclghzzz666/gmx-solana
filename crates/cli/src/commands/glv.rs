use std::ops::Deref;

use gmsol_sdk::solana_utils::solana_sdk::{pubkey::Pubkey, signer::Signer};

/// GLV management commands.
#[derive(Debug, clap::Args)]
pub struct Glv {}

impl super::Command for Glv {
    fn is_client_required(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: super::Context<'_>) -> eyre::Result<()> {
        let payer = ctx.client()?.payer();
        println!("{payer}");
        Ok(())
    }
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub(crate) struct GlvToken {
    /// GLV token address.
    #[arg(long)]
    glv_token: Option<Pubkey>,
    /// Index.
    #[arg(long)]
    index: Option<u16>,
}

impl GlvToken {
    pub(crate) fn address<C: Deref<Target = impl Signer> + Clone>(
        &self,
        client: &gmsol_sdk::Client<C>,
        store: &Pubkey,
    ) -> Pubkey {
        match (self.glv_token, self.index) {
            (Some(address), _) => address,
            (None, Some(index)) => client.find_glv_token_address(store, index),
            (None, None) => unreachable!(),
        }
    }
}
