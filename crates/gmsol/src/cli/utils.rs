use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer},
    RequestBuilder,
};
use eyre::ContextCompat;
use gmsol::store::oracle::find_oracle_address;

#[derive(clap::Args, Clone)]
#[group(required = false, multiple = false, id = "oracle_address")]
pub(crate) struct Oracle {
    #[arg(long, env)]
    oracle: Option<Pubkey>,
    #[arg(long, default_value_t = 0)]
    oracle_index: u8,
}

impl Oracle {
    pub(crate) fn address(&self, store: Option<&Pubkey>) -> gmsol::Result<Pubkey> {
        match self.oracle {
            Some(address) => Ok(address),
            None => Ok(find_oracle_address(
                store.wrap_err("`store` not provided")?,
                self.oracle_index,
            )
            .0),
        }
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
