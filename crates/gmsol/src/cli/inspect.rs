use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use data_store::states::{self};
use exchange::utils::ControllerSeeds;
use eyre::ContextCompat;
use gmsol::{
    pyth::{pull_oracle::receiver::PythReceiverOps, utils::get_merkle_price_updates},
    store::{
        market::find_market_address,
        token_config::{find_token_config_map, get_token_config},
    },
    utils::{self, ComputeBudget},
};

use crate::{utils::Oracle, SharedClient};

#[derive(clap::Args)]
pub(super) struct InspectArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// `DataStore` account.
    DataStore { address: Option<Pubkey> },
    /// `Roles` account.
    Roles { address: Pubkey },
    /// `TokenConfigMap` account.
    TokenConfigMap { address: Option<Pubkey> },
    /// `Market` account.
    Market {
        address: Pubkey,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
        /// Whether to display the market address.
        #[arg(long)]
        show_market_address: bool,
    },
    /// `Deposit` account.
    Deposit { address: Pubkey },
    /// `Withdrawal` account.
    Withdrawal { address: Pubkey },
    /// `Oracle` account.
    Oracle {
        #[command(flatten)]
        oracle: Oracle,
    },
    /// Get the CONTROLLER address.
    Controller,
    /// Get token config.
    TokenConfig { token: Pubkey },
    /// `Order` account.
    Order { address: Pubkey },
    /// `Position` account.
    Position { address: Pubkey },
    /// Watch Pyth Price Updates.
    WatchPyth {
        #[arg(required = true)]
        feed_ids: Vec<String>,
        #[arg(long)]
        post: bool,
    },
    /// Generate Anchor Discriminator with the given name.
    Discriminator { name: String },
}

impl InspectArgs {
    pub(super) async fn run(
        &self,
        client: &SharedClient,
        store: Option<&Pubkey>,
    ) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match &self.command {
            Command::Discriminator { name } => {
                println!("{:?}", crate::utils::generate_discriminator(name));
            }
            Command::DataStore { address } => {
                let address = address
                    .or(store.copied())
                    .wrap_err("missing store address")?;
                println!(
                    "{:#?}",
                    program.account::<states::DataStore>(address).await?
                );
            }
            Command::Roles { address } => {
                println!("{:#?}", program.account::<states::Roles>(*address).await?);
            }
            Command::TokenConfigMap { address } => {
                let address = address
                    .or_else(|| store.as_ref().map(|store| find_token_config_map(store).0))
                    .wrap_err("missing store address")?;
                println!(
                    "{:#?}",
                    program.account::<states::TokenConfigMap>(address).await?
                );
            }
            Command::Market {
                mut address,
                as_market_address,
                show_market_address,
            } => {
                if !as_market_address {
                    address =
                        find_market_address(store.wrap_err("`store` not provided")?, &address).0;
                }
                println!("{:#?}", program.account::<states::Market>(address).await?);
                if *show_market_address {
                    println!("Market address: {address}");
                }
            }
            Command::Deposit { address } => {
                println!("{:#?}", program.account::<states::Deposit>(*address).await?);
            }
            Command::Withdrawal { address } => {
                println!(
                    "{:#?}",
                    program.account::<states::Withdrawal>(*address).await?
                );
            }
            Command::Controller => {
                let controller =
                    ControllerSeeds::find_with_address(store.wrap_err("missing `store` address")?)
                        .1;
                println!("{controller}");
            }
            Command::Oracle { oracle } => {
                let address = oracle.address(store)?;
                println!("{address}");
                println!("{:#?}", program.account::<states::Oracle>(address).await?);
            }
            Command::TokenConfig { token } => {
                let store = store.wrap_err("missing store address")?;
                let config = get_token_config(&program, store, token)
                    .await?
                    .ok_or(gmsol::Error::NotFound)?;
                println!("{config:#?}");
            }
            Command::Order { address } => {
                println!("{:#?}", program.account::<states::Order>(*address).await?);
            }
            Command::Position { address } => {
                println!(
                    "{:#?}",
                    utils::try_deserailize_account::<states::Position>(
                        &program.async_rpc(),
                        address
                    )
                    .await?
                );
            }
            Command::WatchPyth { feed_ids, post } => {
                use futures_util::TryStreamExt;
                use gmsol::pyth::{
                    pull_oracle::WormholeOps, utils, EncodingType, Hermes, PythPullOracleOps,
                };
                use pyth_sdk::Identifier;

                let hermes = Hermes::default();

                let feed_ids = feed_ids
                    .iter()
                    .map(|id| {
                        let hex = id.strip_prefix("0x").unwrap_or(id);
                        Identifier::from_hex(hex).map_err(gmsol::Error::unknown)
                    })
                    .collect::<gmsol::Result<Vec<_>>>()?;

                let stream = hermes
                    .price_updates(feed_ids, Some(EncodingType::Base64))
                    .await?;
                futures_util::pin_mut!(stream);
                while let Some(update) = stream.try_next().await? {
                    tracing::info!("{:#?}", update.parsed());
                    let datas = utils::parse_accumulator_update_datas(&update)?;
                    for data in datas {
                        let proof = &data.proof;
                        let guardian_set_index = utils::get_guardian_set_index(proof)?;
                        tracing::info!("{guardian_set_index}:{proof:?}");
                        if *post {
                            let encoded_vaa = Keypair::new();
                            let draft_vaa = encoded_vaa.pubkey();
                            let vaa = utils::get_vaa_buffer(proof);
                            let wormhole = client.wormhole()?;
                            let pyth = client.pyth()?;

                            tracing::info!("sending txs...");
                            let signature = wormhole
                                .create_encoded_vaa(&encoded_vaa, vaa.len() as u64)
                                .await?
                                .build()
                                .send()
                                .await?;
                            tracing::info!(%draft_vaa, "initialized an encoded vaa account at tx {signature}");

                            let signature = wormhole
                                .write_encoded_vaa(&draft_vaa, 0, vaa)
                                .build()
                                .send()
                                .await?;
                            tracing::info!(%draft_vaa, "written to the encoded vaa account at tx {signature}");

                            let signature = wormhole
                                .verify_encoded_vaa_v1(&draft_vaa, guardian_set_index)
                                .compute_budget(Some(ComputeBudget::default().with_limit(400_000)))
                                .build()
                                .send()
                                .await?;
                            tracing::info!(%draft_vaa, "verified the encoded vaa account at tx {signature}");

                            let updates = get_merkle_price_updates(proof);
                            for update in updates {
                                let price_update = Keypair::new();
                                let price_update_pubkey = price_update.pubkey();
                                let (request, (feed_id, _)) = pyth
                                    .post_price_update(&price_update, update, &draft_vaa)?
                                    .build_with_output();
                                let signature = request.send().await?;
                                tracing::info!(%feed_id, price_update=%price_update_pubkey, "posted a price update at tx {signature}");

                                let signature = pyth
                                    .reclaim_rent(&price_update_pubkey)
                                    .build()
                                    .send()
                                    .await?;
                                tracing::info!(%feed_id, price_update=%price_update_pubkey, "reclaimed rent at tx {signature}");
                            }

                            let signature = wormhole
                                .close_encoded_vaa(&draft_vaa)
                                .build()
                                .send()
                                .await?;

                            tracing::info!(encoded_vaa=%draft_vaa, "closed the encoded vaa account at tx {signature}");
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
