use anchor_client::solana_sdk::pubkey::Pubkey;
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
