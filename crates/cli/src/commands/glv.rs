use std::{ops::Deref, path::PathBuf};

use gmsol_sdk::{
    core::glv::GlvMarketFlag,
    ops::GlvOps,
    programs::gmsol_store::types::UpdateGlvParams,
    serde::StringPubkey,
    solana_utils::solana_sdk::{pubkey::Pubkey, signer::Signer},
    utils::{GmAmount, Value},
};
use indexmap::IndexMap;

use super::utils::{toml_from_file, ToggleValue};

/// GLV management commands.
#[derive(Debug, clap::Args)]
pub struct Glv {
    #[command(flatten)]
    glv_token: GlvToken,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Inititalize a GLV.
    Init {
        #[clap(required = true)]
        market_tokens: Vec<Pubkey>,
    },
    /// Update Config.
    UpdateConfig(UpdateGlvArgs),
    /// Toggle GLV market flag.
    ToggleMarketFlag {
        market_token: Pubkey,
        #[arg(long)]
        flag: GlvMarketFlag,
        #[command(flatten)]
        toggle: ToggleValue,
    },
    /// Update Market Config.
    UpdateMarket {
        market_token: Pubkey,
        #[command(flatten)]
        config: MarketConfig,
    },
    /// Insert Market.
    InsertMarket {
        #[clap(required = true)]
        market_tokens: Vec<Pubkey>,
    },
    /// Remove Market.
    RemoveMarket {
        #[clap(required = true)]
        market_tokens: Vec<Pubkey>,
    },
}

impl super::Command for Glv {
    fn is_client_required(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: super::Context<'_>) -> eyre::Result<()> {
        let selected = &self.glv_token;
        let client = ctx.client()?;
        let store = ctx.store();
        let options = ctx.bundle_options();

        let bundle = match &self.command {
            Command::Init { market_tokens } => {
                let Some(index) = selected.index else {
                    eyre::bail!("must provide --index to init GLV");
                };
                let (rpc, glv_token) =
                    client.initialize_glv(store, index, market_tokens.iter().copied())?;
                if glv_token != selected.address(client, store) {
                    eyre::bail!("the GLV token to be initialized is not the selected one");
                }
                println!("GLV Token: {glv_token}");
                rpc.into_bundle_with_options(options)?
            }
            Command::UpdateConfig(args) => {
                let glv_token = selected.address(client, store);
                match &args.file {
                    Some(file) => {
                        let UpdateGlv { glv, market } = toml_from_file(file)?;

                        let mut bundle = client.bundle_with_options(options);

                        let params: UpdateGlvParams = glv.try_into()?;
                        if params.min_tokens_for_first_deposit.is_some()
                            || params.shift_max_price_impact_factor.is_some()
                            || params.shift_min_interval_secs.is_some()
                            || params.shift_min_value.is_some()
                        {
                            bundle.push(client.update_glv_config(store, &glv_token, params))?;
                        }

                        for (market_token, MarketConfigWithFlag { config, flag }) in market {
                            bundle.push(client.update_glv_market_config(
                                store,
                                &glv_token,
                                &market_token,
                                config.max_amount()?,
                                config.max_value()?,
                            ))?;

                            for (flag, enable) in flag {
                                bundle.push(client.toggle_glv_market_flag(
                                    store,
                                    &glv_token,
                                    &market_token,
                                    flag,
                                    enable,
                                ))?;
                            }
                        }

                        bundle
                    }
                    None => client
                        .update_glv_config(store, &glv_token, args.into())
                        .into_bundle_with_options(options)?,
                }
            }
            Command::ToggleMarketFlag {
                market_token,
                flag,
                toggle,
            } => client
                .toggle_glv_market_flag(
                    store,
                    &selected.address(client, store),
                    market_token,
                    *flag,
                    toggle.is_enable(),
                )
                .into_bundle_with_options(options)?,
            Command::UpdateMarket {
                market_token,
                config,
            } => client
                .update_glv_market_config(
                    store,
                    &selected.address(client, store),
                    market_token,
                    config.max_amount()?,
                    config.max_value()?,
                )
                .into_bundle_with_options(options)?,
            Command::InsertMarket { market_tokens } => {
                let mut bundle = client.bundle_with_options(options);
                let glv_token = selected.address(client, store);
                for market_token in market_tokens {
                    bundle.push(client.insert_glv_market(store, &glv_token, market_token, None))?;
                }
                bundle
            }
            Command::RemoveMarket { market_tokens } => {
                let mut bundle = client.bundle_with_options(options);
                let glv_token = selected.address(client, store);

                for market_token in market_tokens {
                    bundle.push(client.remove_glv_market(store, &glv_token, market_token, None))?;
                }

                bundle
            }
        };

        client.send_or_serialize(bundle).await?;

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

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = true)]
struct UpdateGlvArgs {
    /// Path to the update file (TOML).
    #[arg(long, short)]
    file: Option<PathBuf>,
    /// Minimum amount for the first GLV deposit.
    #[arg(long)]
    min_tokens_for_first_deposit: Option<u64>,
    /// Minimum shift interval seconds.
    #[arg(long)]
    shift_min_interval_secs: Option<u32>,
    /// Maximum price impact factor after shift.
    #[arg(long)]
    shift_max_price_impact_factor: Option<u128>,
    /// Minimum shift value.
    #[arg(long)]
    shift_min_value: Option<u128>,
}

impl<'a> From<&'a UpdateGlvArgs> for UpdateGlvParams {
    fn from(args: &'a UpdateGlvArgs) -> Self {
        Self {
            min_tokens_for_first_deposit: args.min_tokens_for_first_deposit,
            shift_min_interval_secs: args.shift_min_interval_secs,
            shift_max_price_impact_factor: args.shift_max_price_impact_factor,
            shift_min_value: args.shift_min_value,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, clap::Args)]
#[group(required = true, multiple = true)]
struct MarketConfig {
    #[arg(long)]
    max_amount: Option<GmAmount>,
    #[arg(long)]
    max_value: Option<Value>,
}

impl MarketConfig {
    fn max_amount(&self) -> gmsol_sdk::Result<Option<u64>> {
        self.max_amount
            .as_ref()
            .map(|f| f.0.try_into().map_err(gmsol_sdk::Error::custom))
            .transpose()
    }

    fn max_value(&self) -> gmsol_sdk::Result<Option<u128>> {
        self.max_value.as_ref().map(|f| f.to_u128()).transpose()
    }
}

#[serde_with::serde_as]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MarketConfigWithFlag {
    #[serde(flatten)]
    config: MarketConfig,
    #[serde(flatten)]
    #[serde_as(as = "IndexMap<serde_with::DisplayFromStr, _>")]
    flag: IndexMap<GlvMarketFlag, bool>,
}

#[serde_with::serde_as]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GlvConfig {
    min_tokens_for_first_deposit: Option<GmAmount>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    shift_min_interval: Option<humantime::Duration>,
    shift_max_price_impact_factor: Option<Value>,
    shift_min_value: Option<Value>,
}

impl TryFrom<GlvConfig> for UpdateGlvParams {
    type Error = gmsol_sdk::Error;

    fn try_from(config: GlvConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            min_tokens_for_first_deposit: config
                .min_tokens_for_first_deposit
                .map(|f| f.0.try_into().map_err(gmsol_sdk::Error::custom))
                .transpose()?,
            shift_min_interval_secs: config
                .shift_min_interval
                .map(|d| d.as_secs().try_into().map_err(gmsol_sdk::Error::custom))
                .transpose()?,
            shift_max_price_impact_factor: config
                .shift_max_price_impact_factor
                .map(|f| f.to_u128())
                .transpose()?,
            shift_min_value: config.shift_min_value.map(|f| f.to_u128()).transpose()?,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct UpdateGlv {
    #[serde(flatten)]
    glv: GlvConfig,
    #[serde(flatten)]
    market: IndexMap<StringPubkey, MarketConfigWithFlag>,
}
