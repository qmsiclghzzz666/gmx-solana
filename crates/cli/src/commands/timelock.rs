use std::{ops::Deref, time::Duration};

use gmsol_sdk::{
    core::oracle::PriceProviderKind,
    ops::TimelockOps,
    programs::{
        anchor_lang::prelude::Pubkey,
        gmsol_timelock::accounts::{Executor, TimelockConfig},
    },
    solana_utils::solana_sdk::signer::Signer,
    utils::zero_copy::ZeroCopy,
};

/// Timelock commands.
#[derive(Debug, clap::Args)]
pub struct Timelock {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Get current config.
    Config,
    /// Get executor.
    Executor { role: String },
    /// Get executor wallet.
    ExecutorWallet { role: String },
    /// Initialize timelock config.
    InitConfig {
        #[arg(long, default_value = "1d", value_parser = humantime::parse_duration)]
        initial_delay: Duration,
    },
    /// Increase timelock delay.
    IncreaseDelay {
        #[arg(value_parser = humantime::parse_duration)]
        delta: Duration,
    },
    /// Init executor.
    InitExecutor { role: String },
    /// Approve a timelocked instruction.
    Approve {
        buffers: Vec<Pubkey>,
        #[arg(long)]
        role: Option<String>,
    },
    /// Cancel a timelocked instruction.
    Cancel {
        buffers: Vec<Pubkey>,
        #[arg(long)]
        executor: Option<Pubkey>,
        #[arg(long)]
        rent_receiver: Option<Pubkey>,
    },
    /// Execute a timelocked instruction.
    Execute { buffers: Vec<Pubkey> },
    /// Revoke role.
    RevokeRole {
        /// User.
        authority: Pubkey,
        /// Role.
        role: String,
    },
    /// Set expected price provider.
    SetExpectedPriceProvider {
        /// Token.
        token: Pubkey,
        /// New Price Provider.
        provider: PriceProviderKind,
        /// Token map.
        #[arg(long)]
        token_map: Option<Pubkey>,
    },
}

impl super::Command for Timelock {
    fn is_client_required(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: super::Context<'_>) -> eyre::Result<()> {
        let client = ctx.client()?;
        let options = ctx.bundle_options();
        let store = &ctx.store;
        let bundle = match &self.command {
            Command::Config => {
                let config_address = client.find_timelock_config_address(store);
                let config = client
                    .account::<ZeroCopy<TimelockConfig>>(&config_address)
                    .await?;
                match config {
                    Some(config) => {
                        let config = config.0;
                        println!("Address: {config_address}");
                        println!("Delay: {}s", config.delay);
                    }
                    None => {
                        println!("Not initialized");
                    }
                }
                return Ok(());
            }
            Command::Executor { role } => {
                let executor = get_and_validate_executor_address(client, store, role).await?;
                let wallet = client.find_executor_wallet_address(&executor);
                println!("Executor: {executor}");
                println!("Wallet: {wallet}");
                return Ok(());
            }
            Command::ExecutorWallet { role } => {
                let executor = get_and_validate_executor_address(client, store, role).await?;
                let wallet = client.find_executor_wallet_address(&executor);
                println!("{wallet}");
                return Ok(());
            }
            Command::InitConfig { initial_delay } => {
                let (rpc, config) = client
                    .initialize_timelock_config(store, initial_delay.as_secs().try_into()?)
                    .swap_output(());
                println!("{config}");
                rpc.into_bundle_with_options(options)?
            }
            Command::IncreaseDelay { delta } => client
                .increase_timelock_delay(store, delta.as_secs().try_into()?)
                .into_bundle_with_options(options)?,
            Command::InitExecutor { role } => {
                let (rpc, executor) = client.initialize_executor(store, role)?.swap_output(());
                println!("{executor}");
                rpc.into_bundle_with_options(options)?
            }
            Command::Approve { buffers, role } => client
                .approve_timelocked_instructions(
                    store,
                    buffers.iter().copied(),
                    role.as_ref().map(|s| s.as_str()),
                )
                .await?
                .into_bundle_with_options(options)?,
            Command::Cancel {
                buffers,
                executor,
                rent_receiver,
            } => client
                .cancel_timelocked_instructions(
                    store,
                    buffers.iter().copied(),
                    executor.as_ref(),
                    rent_receiver.as_ref(),
                )
                .await?
                .into_bundle_with_options(options)?,
            Command::Execute { buffers } => {
                let mut txns = client.bundle_with_options(options);
                for buffer in buffers {
                    let rpc = client
                        .execute_timelocked_instruction(store, buffer, None)
                        .await?;
                    txns.push(rpc)?;
                }
                txns
            }
            Command::RevokeRole { authority, role } => client
                .timelock_bypassed_revoke_role(store, role, authority)
                .into_bundle_with_options(options)?,
            Command::SetExpectedPriceProvider {
                token,
                provider,
                token_map,
            } => {
                let token_map = match *token_map {
                    Some(token_map) => token_map,
                    None => client
                        .authorized_token_map_address(store)
                        .await?
                        .ok_or(gmsol_sdk::Error::NotFound)?,
                };
                client
                    .timelock_bypassed_set_epxected_price_provider(
                        store, &token_map, token, *provider,
                    )
                    .into_bundle_with_options(options)?
            }
        };
        client.send_or_serialize(bundle).await?;
        Ok(())
    }
}

async fn get_and_validate_executor_address<C: Deref<Target = impl Signer> + Clone>(
    client: &gmsol_sdk::Client<C>,
    store: &Pubkey,
    role: &str,
) -> gmsol_sdk::Result<Pubkey> {
    let executor = client.find_executor_address(store, role)?;
    let account = client
        .account::<ZeroCopy<Executor>>(&executor)
        .await?
        .ok_or(gmsol_sdk::Error::NotFound)?;
    if account.0.role_name()? == role {
        Ok(executor)
    } else {
        Err(gmsol_sdk::Error::custom(format!(
            "invalid executor account found: {executor}"
        )))
    }
}
