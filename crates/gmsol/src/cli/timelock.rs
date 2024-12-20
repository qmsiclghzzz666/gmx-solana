use crate::GMSOLClient;
use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::{timelock::TimelockOps, utils::ZeroCopy};
use gmsol_timelock::states::Executor;

#[derive(clap::Args)]
pub(super) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Get executor.
    Executor { role: String },
    /// Initialize timelock config.
    InitConfig {
        #[arg(long, default_value_t = 86400)]
        initial_delay: u32,
    },
    /// Increase timelock delay.
    IncreaseDelay { delta: u32 },
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
}

impl Args {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
        skip_preflight: bool,
    ) -> gmsol::Result<()> {
        let req = match &self.command {
            Command::Executor { role } => {
                let executor = client.find_executor_address(store, role)?;
                let account = client
                    .account::<ZeroCopy<Executor>>(&executor)
                    .await?
                    .ok_or(gmsol::Error::NotFound)?;
                if account.0.role_name()? != role {
                    return Err(gmsol::Error::invalid_argument(format!(
                        "invalid executor account found: {executor}"
                    )));
                }
                println!("{executor}");
                return Ok(());
            }
            Command::InitConfig { initial_delay } => {
                let (rpc, config) = client
                    .initialize_timelock_config(store, *initial_delay)
                    .swap_output(());
                println!("{config}");
                rpc
            }
            Command::IncreaseDelay { delta } => client.increase_timelock_delay(store, *delta),
            Command::InitExecutor { role } => {
                let (rpc, executor) = client.initialize_executor(store, role)?.swap_output(());
                println!("{executor}");
                rpc
            }
            Command::Approve { buffers, role } => {
                client
                    .approve_timelocked_instructions(
                        store,
                        buffers.iter().copied(),
                        role.as_ref().map(|s| s.as_str()),
                    )
                    .await?
            }
            Command::Cancel { buffers, executor } => {
                client
                    .cancel_timelocked_instructions(
                        store,
                        buffers.iter().copied(),
                        executor.as_ref(),
                    )
                    .await?
            }
            Command::Execute { buffers } => {
                let mut txns = client.transaction();
                for buffer in buffers {
                    let rpc = client
                        .execute_timelocked_instruction(store, buffer, None)
                        .await?;
                    txns.push(rpc)?;
                }
                return crate::utils::send_or_serialize_transactions(
                    txns,
                    serialize_only,
                    skip_preflight,
                    |signatures, error| {
                        match error {
                            Some(err) => {
                                tracing::error!(%err, "successful txns: {signatures:#?}");
                            }
                            None => {
                                tracing::info!("successful txns: {signatures:#?}");
                            }
                        }
                        Ok(())
                    },
                )
                .await;
            }
            Command::RevokeRole { authority, role } => {
                client.timelock_bypassed_revoke_role(store, role, authority)
            }
        };
        crate::utils::send_or_serialize_rpc(
            store,
            req,
            None,
            serialize_only,
            skip_preflight,
            |signature| {
                tracing::info!("{signature}");
                Ok(())
            },
        )
        .await
    }
}
