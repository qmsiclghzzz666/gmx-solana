use crate::{utils::ToggleValue, GMSOLClient, TimelockCtx};
use gmsol::{store::glv::GlvOps, types::glv::GlvMarketFlag};
use solana_sdk::pubkey::Pubkey;

#[derive(clap::Args)]
pub(super) struct Args {
    #[command(flatten)]
    glv_token: GlvToken,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Inititalize a GLV.
    Init {
        #[clap(required = true)]
        market_tokens: Vec<Pubkey>,
    },
    /// Toggle GLV market flag.
    Toggle {
        market_token: Pubkey,
        #[arg(long)]
        flag: GlvMarketFlag,
        #[command(flatten)]
        toggle: ToggleValue,
    },
    /// Update.
    Update {
        market_token: Pubkey,
        #[command(flatten)]
        config: Config,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct GlvToken {
    /// GLV token address.
    #[arg(long)]
    glv_token: Option<Pubkey>,
    /// Index.
    #[arg(long)]
    index: Option<u8>,
}

#[derive(clap::Args)]
#[group(required = true, multiple = true)]
struct Config {
    #[arg(long)]
    max_amount: Option<u64>,
    #[arg(long)]
    max_value: Option<u128>,
}

impl GlvToken {
    fn address(&self, client: &GMSOLClient, store: &Pubkey) -> Pubkey {
        match (self.glv_token, self.index) {
            (Some(address), _) => address,
            (None, Some(index)) => client.find_glv_token_address(store, index),
            (None, None) => unreachable!(),
        }
    }
}

impl Args {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        timelock: Option<TimelockCtx<'_>>,
        serialize_only: bool,
        skip_preflight: bool,
    ) -> gmsol::Result<()> {
        let selected = &self.glv_token;
        let rpc = match &self.command {
            Command::Init { market_tokens } => {
                let Some(index) = selected.index else {
                    return Err(gmsol::Error::invalid_argument(
                        "must provide index to init GLV",
                    ));
                };
                let (rpc, glv_token) =
                    client.initialize_glv(store, index, market_tokens.iter().copied())?;
                if glv_token != selected.address(client, store) {
                    return Err(gmsol::Error::invalid_argument(
                        "the GLV token to be initialized is not the selected one",
                    ));
                }
                println!("{glv_token}");
                rpc
            }
            Command::Toggle {
                market_token,
                flag,
                toggle,
            } => client.toggle_glv_market_flag(
                store,
                &selected.address(client, store),
                market_token,
                *flag,
                toggle.is_enable(),
            ),
            Command::Update {
                market_token,
                config,
            } => client.update_glv_market_config(
                store,
                &selected.address(client, store),
                market_token,
                config.max_amount,
                config.max_value,
            ),
        };

        crate::utils::send_or_serialize_rpc(
            store,
            rpc,
            timelock,
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
