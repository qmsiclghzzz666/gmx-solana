use std::num::NonZeroUsize;

use eyre::OptionExt;
use futures_util::StreamExt;
use gmsol_sdk::programs::anchor_lang::prelude::Pubkey;

/// Inspects protocol data.
#[derive(Debug, clap::Args)]
pub struct Inspect {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Print protocol addresses.
    Address { kind: AddressKind },
    /// Inspect an account.
    Account { address: Pubkey },
    /// Inspect events that related to the given account.
    Events {
        address: Pubkey,
        #[arg(long)]
        limit: Option<NonZeroUsize>,
    },
    /// Inspect Timelocked instructions.
    Tld {
        #[clap(long)]
        raw: bool,
        #[arg(required = true)]
        addresses: Vec<Pubkey>,
    },
    #[cfg(feature = "squads")]
    Squads {
        vault_transaction_address: Pubkey,
        #[clap(long)]
        raw: bool,
    },
    /// Inspect Chainlink feed IDs.
    #[cfg(feature = "chainlink")]
    Chainlink {
        #[arg(long)]
        testnet: bool,
        #[arg(long, short)]
        decode: bool,
        #[arg(long, short)]
        watch: bool,
        #[arg(required = true)]
        feed_ids: Vec<String>,
    },
    /// Inspect Pyth feed IDs.
    #[cfg(feature = "pyth")]
    Pyth {
        #[arg(long, short)]
        watch: bool,
        #[arg(required = true)]
        feed_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum AddressKind {
    /// Event authority.
    EventAuthority,
}

impl super::Command for Inspect {
    fn is_client_required(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: super::Context<'_>) -> eyre::Result<()> {
        let client = ctx.client()?;
        let store = ctx.store();

        match &self.command {
            Command::Address { kind } => {
                let address = match kind {
                    AddressKind::EventAuthority => client.store_event_authority(),
                };
                println!("{address}");
            }
            Command::Account { address } => {
                use gmsol_sdk::{
                    decode::{
                        decoder::AccountAccessDecoder, gmsol::programs::GMSOLAccountData, Decode,
                    },
                    programs::{
                        gmsol_competition::{
                            utils::Account as CompetitionAccount, ID as COMPETITION_PROGRAM_ID,
                        },
                        gmsol_timelock::utils::Account as TimelockAccount,
                        gmsol_treasury::utils::Account as TreausryAccount,
                    },
                    solana_utils::utils::WithSlot,
                    utils::decode::KeyedAccount,
                };

                let res = client
                    .raw_account_with_config(address, Default::default())
                    .await?;
                let slot = res.slot();
                let account = res.into_value().ok_or_eyre("account does not exist")?;
                if account.owner == *client.store_program_id() {
                    let account = KeyedAccount {
                        pubkey: *address,
                        account: WithSlot::new(slot, account),
                    };
                    let decoder = AccountAccessDecoder::new(account);
                    let decoded = GMSOLAccountData::decode(decoder)?;
                    println!("{decoded:#?}");
                } else if account.owner == *client.treasury_program_id() {
                    let account = TreausryAccount::try_from_bytes(&account.data)?;
                    match account {
                        TreausryAccount::Config(a) => println!("{a:#?}"),
                        TreausryAccount::GtBank(a) => println!("{a:#?}"),
                        TreausryAccount::GtExchange(a) => println!("{a:#?}"),
                        TreausryAccount::GtExchangeVault(a) => println!("{a:#?}"),
                        TreausryAccount::Oracle(a) => println!("{a:#?}"),
                        TreausryAccount::Store(a) => println!("{a:#?}"),
                        TreausryAccount::TreasuryVaultConfig(a) => println!("{a:#?}"),
                    }
                } else if account.owner == *client.timelock_program_id() {
                    let account = TimelockAccount::try_from_bytes(&account.data)?;
                    match account {
                        TimelockAccount::Executor(a) => println!("{a:#?}"),
                        TimelockAccount::InstructionHeader(a) => println!("{a:#?}"),
                        TimelockAccount::Store(a) => println!("{a:#?}"),
                        TimelockAccount::TimelockConfig(a) => println!("{a:#?}"),
                    }
                } else if account.owner == COMPETITION_PROGRAM_ID {
                    let account = CompetitionAccount::try_from_bytes(&account.data)?;
                    match account {
                        CompetitionAccount::Competition(a) => println!("{a:#?}"),
                        CompetitionAccount::Participant(a) => println!("{a:#?}"),
                        CompetitionAccount::TradeData(a) => println!("{a:#?}"),
                    }
                }
            }
            Command::Events { address, limit } => {
                let stream = match limit {
                    Some(limit) => client
                        .historical_store_cpi_events(address, None)
                        .await?
                        .take(limit.get())
                        .left_stream(),
                    None => client
                        .historical_store_cpi_events(address, None)
                        .await?
                        .right_stream(),
                };
                futures_util::pin_mut!(stream);
                while let Some(res) = stream.next().await {
                    match res {
                        Ok(events) => {
                            println!("{events:#?}");
                        }
                        Err(err) => {
                            tracing::error!(%err, "stream error");
                        }
                    }
                }
            }
            Command::Tld { raw, addresses } => {
                use gmsol_sdk::{
                    core::instruction::{InstructionAccess, InstructionFlag},
                    programs::gmsol_timelock::accounts::TimelockConfig,
                    solana_utils::{
                        solana_sdk::message::{Message, VersionedMessage},
                        utils::inspect_transaction,
                    },
                    utils::zero_copy::ZeroCopy,
                };

                let config = client.find_timelock_config_address(store);
                let delay = client
                    .account::<ZeroCopy<TimelockConfig>>(&config)
                    .await?
                    .ok_or(gmsol_sdk::Error::NotFound)?
                    .0
                    .delay;
                let delay = time::Duration::seconds(delay as i64);

                let mut instructions = Vec::with_capacity(addresses.len());

                for (idx, address) in addresses.iter().enumerate() {
                    let buffer = client
                        .instruction_buffer(address)
                        .await?
                        .ok_or(gmsol_sdk::Error::NotFound)?;

                    let status = if buffer.header.flags.get_flag(InstructionFlag::Approved) {
                        let approved_at =
                            time::OffsetDateTime::from_unix_timestamp(buffer.header.approved_at)
                                .map_err(gmsol_sdk::Error::custom)?;
                        let executable_at = approved_at.saturating_add(delay);
                        let now = time::OffsetDateTime::now_utc();
                        let delta = executable_at - now;
                        if delta.is_positive() {
                            format!("executable in {delta}")
                        } else {
                            "executable".to_string()
                        }
                    } else {
                        "is not approved".to_string()
                    };
                    tracing::info!("[{idx}] {address}: {status}");

                    instructions.push(
                        buffer
                            .to_instruction(true)
                            .map_err(gmsol_sdk::Error::custom)?,
                    );
                }

                let message = Message::new(&instructions, Some(&client.payer()));
                println!(
                    "{}",
                    inspect_transaction(
                        &VersionedMessage::Legacy(message),
                        Some(client.cluster()),
                        *raw,
                    )
                );
            }
            #[cfg(feature = "squads")]
            Command::Squads {
                vault_transaction_address,
                raw,
            } => {
                use gmsol_sdk::{
                    client::squads::{SquadsProposal, SquadsVaultTransaction},
                    solana_utils::{
                        solana_sdk::message::VersionedMessage, utils::inspect_transaction,
                    },
                    squads::{get_proposal_pda, get_vault_pda},
                };

                let vault_transaction = client
                    .account::<SquadsVaultTransaction>(vault_transaction_address)
                    .await?
                    .ok_or(gmsol_sdk::Error::NotFound)?;

                let multisig = &vault_transaction.multisig;
                let proposal_pubkey = get_proposal_pda(multisig, vault_transaction.index, None).0;

                let proposal = client
                    .account::<SquadsProposal>(&proposal_pubkey)
                    .await?
                    .ok_or(gmsol_sdk::Error::NotFound)?;

                let message = vault_transaction.to_message();

                println!("Transaction Index: {}", vault_transaction.index);
                println!("Status: {:?}", proposal.status);
                println!(
                    "Results: approved {} - rejected {}",
                    proposal.approved.len(),
                    proposal.rejected.len()
                );
                println!("Multisig: {multisig}");
                println!(
                    "Vault: {}",
                    get_vault_pda(multisig, vault_transaction.vault_index, None).0
                );
                println!("Proposal: {proposal_pubkey}");
                println!("Creator: {}", vault_transaction.creator);
                println!(
                    "Inspector: {}",
                    inspect_transaction(
                        &VersionedMessage::V0(message),
                        Some(client.cluster()),
                        *raw
                    )
                );
            }
            #[cfg(feature = "chainlink")]
            Command::Chainlink {
                testnet,
                decode,
                watch,
                feed_ids,
            } => {
                use gmsol_sdk::client::chainlink::{
                    gmsol_chainlink_datastreams::report::Report, Client,
                };
                use time::OffsetDateTime;

                fn display_report(report: &Report) -> gmsol_sdk::Result<String> {
                    Ok(format!("{:#?}", report))
                }

                let client = if *testnet {
                    Client::from_testnet_defaults()?
                } else {
                    Client::from_defaults()?
                };

                let feed_ids = feed_ids.iter().map(|s| s.as_str());

                if *watch {
                    let stream = client.subscribe(feed_ids).await?;
                    futures_util::pin_mut!(stream);
                    while let Some(report) = stream.next().await {
                        match report {
                            Ok(report) => {
                                if *decode {
                                    println!("{}", display_report(&report.decode()?)?);
                                } else {
                                    println!("{report:#?}");
                                }
                            }
                            Err(err) => {
                                tracing::error!(%err, "receive error");
                            }
                        }
                    }
                } else {
                    let ts = OffsetDateTime::now_utc();
                    let reports = client.bulk_report(feed_ids, ts).await?;
                    for report in reports.iter() {
                        if *decode {
                            println!("{}", display_report(&report.decode()?)?);
                        } else {
                            println!("{report:#?}");
                        }
                    }
                }
            }
            #[cfg(feature = "pyth")]
            Command::Pyth { watch, feed_ids } => {
                use futures_util::TryStreamExt;
                use gmsol_sdk::client::pyth::{
                    pull_oracle::hermes::Identifier, EncodingType, Hermes,
                };

                fn parse_feed_ids(feed_ids: &[String]) -> gmsol_sdk::Result<Vec<Identifier>> {
                    let feed_ids = feed_ids
                        .iter()
                        .map(|id| {
                            let hex = id.strip_prefix("0x").unwrap_or(id);
                            Identifier::from_hex(hex).map_err(gmsol_sdk::Error::custom)
                        })
                        .collect::<gmsol_sdk::Result<Vec<_>>>()?;
                    Ok(feed_ids)
                }

                let hermes = Hermes::default();
                let feed_ids = parse_feed_ids(feed_ids)?;

                if *watch {
                    let stream = hermes
                        .price_updates(&feed_ids, Some(EncodingType::Base64))
                        .await?;
                    futures_util::pin_mut!(stream);
                    while let Some(update) = stream.try_next().await? {
                        println!("{:#?}", update.parsed());
                    }
                } else {
                    let update = hermes
                        .latest_price_updates(&feed_ids, Some(EncodingType::Base64))
                        .await?;
                    println!("{:#?}", update.parsed());
                }
            }
        }
        Ok(())
    }
}
