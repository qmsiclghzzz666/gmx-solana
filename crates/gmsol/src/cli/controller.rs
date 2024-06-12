use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{AddressKey, Amount, AmountKey, Factor, FactorKey};
use gmsol::store::{config::ConfigOps, oracle::OracleOps};

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct ControllerArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize a [`Oracle`](data_store::states::Oracle) account.
    InitializeOracle { index: u8 },
    /// Insert an amount to the config.
    InsertAmount {
        amount: Amount,
        #[arg(long, short)]
        key: AmountKey,
    },
    /// Insert a factor to the config.
    InsertFactor {
        factor: Factor,
        #[arg(long, short)]
        key: FactorKey,
    },
    /// Insert an address to the config.
    InsertAddress {
        address: Pubkey,
        #[arg(long, short)]
        key: AddressKey,
    },
}

impl ControllerArgs {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.command {
            Command::InitializeOracle { index } => {
                let (request, oracle) = client.initialize_oracle(store, *index);
                crate::utils::send_or_serialize(request, serialize_only, |signature| {
                    println!("created oracle {oracle} at tx {signature}");
                    Ok(())
                })
                .await?;
            }
            Command::InsertAmount { amount, key } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_amount_by_key(store, *key, amount)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertFactor { factor, key } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_factor_by_key(store, *key, factor)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertAddress { address, key } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_address_by_key(store, *key, address)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}
