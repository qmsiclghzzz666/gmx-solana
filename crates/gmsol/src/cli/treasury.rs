use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_spl::{associated_token::get_associated_token_address, token_interface::TokenAccount};
use gmsol::{
    client::SystemProgramOps,
    pyth::{pull_oracle::PythPullOracleWithHermes, PythPullOracle},
    treasury::{CreateTreasurySwapOptions, TreasuryOps},
    utils::builder::{MakeTransactionBuilder, WithPullOracle},
};
use gmsol_model::{BalanceExt, BaseMarket};
use gmsol_treasury::states::treasury::TokenFlag;

use crate::{
    utils::{SelectGtExchangeVault, Side},
    GMSOLClient, TimelockCtx,
};

#[derive(clap::Args)]
pub(super) struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize Global Config.
    InitConfig,
    /// Initialize Treasury.
    InitTreasury { index: u8 },
    /// Set treasury.
    SetTreasury { treasury_config: Pubkey },
    /// Set GT factor.
    SetGtFactor { factor: u128 },
    /// Insert token to the treasury.
    InsertToken { token: Pubkey },
    /// Toggle token flag.
    ToggleTokenFlag {
        token: Pubkey,
        #[arg(requires = "toggle")]
        flag: TokenFlag,
        /// Enable the given flag.
        #[arg(long, group = "toggle")]
        enable: bool,
        /// Disable the given flag.
        #[arg(long, group = "toggle")]
        disable: bool,
    },
    /// Set referral reward factors.
    SetReferralReward { factors: Vec<u128> },
    /// Set esGT receiver.
    SetEsgtReceiver { esgt_receiver: Pubkey },
    /// Claim fees.
    ClaimFees {
        market_token: Pubkey,
        #[arg(long)]
        side: Side,
        #[arg(long)]
        deposit: bool,
        #[arg(long)]
        token_program_id: Option<Pubkey>,
    },
    /// Deposit into treasury vault.
    DepositToTreasury {
        token_mint: Pubkey,
        #[arg(long)]
        token_program_id: Option<Pubkey>,
        #[arg(long)]
        no_claim_swap_vault: bool,
    },
    /// Confirm GT buyback.
    ConfirmGtBuyback {
        #[clap(flatten)]
        gt_exchange_vault: SelectGtExchangeVault,
        #[arg(long)]
        oracle: Pubkey,
    },
    /// Sync GT bank.
    SyncGtBank {
        token_mint: Pubkey,
        #[clap(flatten)]
        gt_exchange_vault: SelectGtExchangeVault,
        #[arg(long)]
        token_program_id: Option<Pubkey>,
    },
    /// Create Swap.
    CreateSwap {
        market_token: Pubkey,
        #[arg(long, short = 'i')]
        swap_in: Pubkey,
        #[arg(long, short = 'o')]
        swap_out: Pubkey,
        /// Swap in amount.
        #[arg(long, short)]
        amount: Option<u64>,
        #[arg(long)]
        min_output_amount: Option<u64>,
        /// Extra swap paths.
        #[arg(long, short = 's', action = clap::ArgAction::Append)]
        extra_swap_path: Vec<Pubkey>,
        /// Fund the swap owner.
        #[arg(long, value_name = "LAMPORTS")]
        fund: Option<u64>,
    },
    /// Cancel Swap.
    CancelSwap { order: Pubkey },
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
        let req = match &self.command {
            Command::InitConfig => {
                let (rpc, config) = client.initialize_config(store).swap_output(());
                println!("{config}");
                rpc
            }
            Command::InitTreasury { index } => {
                let (rpc, address) = client.initialize_treasury(store, *index).swap_output(());
                println!("{address}");
                rpc
            }
            Command::SetTreasury { treasury_config } => client.set_treasury(store, treasury_config),
            Command::SetGtFactor { factor } => client.set_gt_factor(store, *factor)?,
            Command::InsertToken { token } => {
                client.insert_token_to_treasury(store, None, token).await?
            }
            Command::ToggleTokenFlag {
                token,
                flag,
                enable,
                disable,
            } => {
                assert!(*enable != *disable);
                let value = *enable;
                client
                    .toggle_token_flag(store, None, token, *flag, value)
                    .await?
            }
            Command::SetReferralReward { factors } => {
                if factors.is_empty() {
                    return Err(gmsol::Error::invalid_argument("factors must be provided"));
                }
                client.set_referral_reward(store, factors.clone())
            }
            Command::SetEsgtReceiver { esgt_receiver } => {
                client.set_esgt_receiver(store, esgt_receiver)
            }
            Command::ClaimFees {
                market_token,
                side,
                deposit,
                token_program_id,
            } => {
                let market = client.find_market_address(store, market_token);
                let market = client.market(&market).await?;
                let amount = market.claimable_fee_pool()?.amount(side.is_long())?;
                if amount == 0 {
                    return Err(gmsol::Error::invalid_argument(
                        "no claimable fees for this side",
                    ));
                }
                let token_mint = market.meta().pnl_token(side.is_long());
                let claim = client.claim_fees_to_receiver_vault(store, market_token, &token_mint);

                if *deposit {
                    let store_account = client.store(store).await?;
                    let time_window = store_account.gt().exchange_time_window();
                    let (deposit, gt_exchange_vault) = client
                        .deposit_into_treasury_valut(
                            store,
                            None,
                            &token_mint,
                            token_program_id.as_ref(),
                            time_window,
                        )
                        .await?
                        .swap_output(());
                    println!("{gt_exchange_vault}");
                    claim.merge(deposit)
                } else {
                    claim
                }
            }
            Command::DepositToTreasury {
                token_mint,
                token_program_id,
                no_claim_swap_vault,
            } => {
                let store_account = client.store(store).await?;
                let time_window = store_account.gt().exchange_time_window();

                let (rpc, gt_exchange_vault) = client
                    .deposit_into_treasury_valut(
                        store,
                        None,
                        token_mint,
                        token_program_id.as_ref(),
                        time_window,
                    )
                    .await?
                    .swap_output(());
                println!("{gt_exchange_vault}");

                if !*no_claim_swap_vault {
                    let claim = client.claim_treasury_swapped_tokens(
                        store,
                        token_mint,
                        token_program_id.as_ref(),
                    );
                    claim.merge(rpc)
                } else {
                    rpc
                }
            }
            Command::ConfirmGtBuyback {
                gt_exchange_vault,
                oracle,
            } => {
                let gt_exchange_vault = gt_exchange_vault.get(store, client).await?;
                let builder = client.confirm_gt_buyback(store, &gt_exchange_vault, oracle);
                // TODO: add support for chainlink.
                let pyth = PythPullOracleWithHermes::from_parts(
                    client,
                    Default::default(),
                    PythPullOracle::try_new(client)?,
                );
                let txns = WithPullOracle::new(&pyth, builder).await?.build().await?;

                return crate::utils::send_or_serialize_transactions(
                    txns,
                    serialize_only,
                    skip_preflight,
                    |signatures, error| {
                        match error {
                            Some(err) => {
                                tracing::error!(%err, "success txns: {signatures:#?}");
                            }
                            None => {
                                tracing::info!("success txns: {signatures:#?}");
                            }
                        }
                        Ok(())
                    },
                )
                .await;
            }
            Command::SyncGtBank {
                gt_exchange_vault,
                token_mint,
                token_program_id,
            } => {
                let gt_exchange_vault = gt_exchange_vault.get(store, client).await?;
                client
                    .sync_gt_bank(
                        store,
                        None,
                        &gt_exchange_vault,
                        token_mint,
                        token_program_id.as_ref(),
                    )
                    .await?
            }
            Command::CreateSwap {
                market_token,
                swap_in,
                swap_out,
                amount,
                min_output_amount,
                extra_swap_path,
                fund,
            } => {
                let config = client.find_config_address(store);
                let amount = match amount {
                    Some(amount) => *amount,
                    None => {
                        let vault = get_associated_token_address(&config, swap_in);
                        let account =
                            client
                                .account::<TokenAccount>(&vault)
                                .await?
                                .ok_or_else(|| {
                                    gmsol::Error::invalid_argument(
                                        "vault account is not initialized",
                                    )
                                })?;
                        account.amount
                    }
                };
                let (rpc, order) = client
                    .create_treasury_swap(
                        store,
                        market_token,
                        swap_in,
                        swap_out,
                        amount,
                        CreateTreasurySwapOptions {
                            swap_path: extra_swap_path.clone(),
                            min_swap_out_amount: *min_output_amount,
                            ..Default::default()
                        },
                    )
                    .await?
                    .swap_output(());
                println!("{order}");
                if let Some(lamports) = fund {
                    let swap_owner = client.find_treasury_swap_owner_address(&config);
                    let fund = client.transfer(&swap_owner, *lamports)?;
                    fund.merge(rpc)
                } else {
                    rpc
                }
            }
            Command::CancelSwap { order } => {
                client.cancel_treasury_swap(store, order, None).await?
            }
        };
        crate::utils::send_or_serialize_rpc(
            store,
            req,
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
