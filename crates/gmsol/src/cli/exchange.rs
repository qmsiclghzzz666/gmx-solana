use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{Market, Position};
use gmsol::{exchange::ExchangeOps, utils};

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct ExchangeArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Create a deposit.
    CreateDeposit {
        /// The address of the market token of the Market to deposit into.
        market_token: Pubkey,
        /// Extra execution fee allowed to use.
        #[arg(long, short, default_value_t = 0)]
        extra_execution_fee: u64,
        /// The token account to receive the minted market tokens.
        ///
        /// Defaults to use assciated token account.
        #[arg(long, short)]
        receiver: Option<Pubkey>,
        /// Minimum amount of market tokens to mint.
        #[arg(long, default_value_t = 0)]
        min_amount: u64,
        /// The initial long token.
        #[arg(long, requires = "long_token_amount")]
        long_token: Option<Pubkey>,
        /// The initial short token.
        #[arg(long, requires = "short_token_amount")]
        short_token: Option<Pubkey>,
        /// The initial long token account.
        #[arg(long)]
        long_token_account: Option<Pubkey>,
        /// The initial short token account.
        #[arg(long)]
        short_token_account: Option<Pubkey>,
        /// The initial long token amount.
        #[arg(long, default_value_t = 0)]
        long_token_amount: u64,
        /// The initial short token amount.
        #[arg(long, default_value_t = 0)]
        short_token_amount: u64,
        /// Swap paths for long token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        long_swap: Vec<Pubkey>,
        /// Swap paths for short token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        short_swap: Vec<Pubkey>,
    },
    /// Cancel a deposit.
    CancelDeposit {
        /// The address of the deposit to cancel.
        deposit: Pubkey,
    },
    /// Create a withdrawal.
    CreateWithdrawal {
        /// The address of the market token of the Market to withdraw from.
        market_token: Pubkey,
        /// Extra execution fee allowed to use.
        #[arg(long, short, default_value_t = 0)]
        extra_execution_fee: u64,
        /// The amount of market tokens to burn.
        #[arg(long)]
        amount: u64,
        /// Final long token.
        #[arg(long)]
        long_token: Option<Pubkey>,
        /// Final short token.
        #[arg(long)]
        short_token: Option<Pubkey>,
        /// The market token account to use.
        #[arg(long)]
        market_token_account: Option<Pubkey>,
        /// The final long token account.
        #[arg(long)]
        long_token_account: Option<Pubkey>,
        /// The final short token account.
        #[arg(long)]
        short_token_account: Option<Pubkey>,
        /// Minimal amount of final long tokens to withdraw.
        #[arg(long, default_value_t = 0)]
        min_long_token_amount: u64,
        /// Minimal amount of final short tokens to withdraw.
        #[arg(long, default_value_t = 0)]
        min_short_token_amount: u64,
        /// Swap paths for long token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        long_swap: Vec<Pubkey>,
        /// Swap paths for short token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        short_swap: Vec<Pubkey>,
    },
    /// Cancel a withdrawal.
    CancelWithdrawal {
        /// The address of the withdrawal to cancel.
        withdrawal: Pubkey,
    },
    /// Create a market increase order.
    MarketIncrease {
        /// The address of the market token of the position's market.
        market_token: Pubkey,
        /// Whether the collateral is long token.
        #[arg(long)]
        collateral_side: Side,
        /// Initial collateral token.
        #[arg(long, short = 'c')]
        initial_collateral_token: Option<Pubkey>,
        /// Initial collateral token account.
        #[arg(long, requires = "initial_collateral_token")]
        initial_collateral_token_account: Option<Pubkey>,
        /// Collateral amount.
        #[arg(long, short = 'a')]
        initial_collateral_token_amount: u64,
        /// Position side.
        #[arg(long)]
        side: Side,
        /// Position increment size in usd.
        #[arg(long)]
        size: u128,
        /// Swap paths for collateral token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
    },
    /// Create a market decrese order.
    MarketDecrease {
        /// The address of the market token of the position's market.
        market_token: Pubkey,
        /// Whether the collateral is long token.
        #[arg(long)]
        collateral_side: Side,
        /// Collateral withdrawal amount.
        #[arg(long, short = 'a', default_value_t = 0)]
        collateral_withdrawal_amount: u64,
        /// Position side.
        #[arg(long)]
        side: Side,
        /// Position decrement size in usd.
        #[arg(long, default_value_t = 0)]
        size: u128,
        /// Final output token.
        #[arg(long, short = 'o')]
        final_output_token: Option<Pubkey>,
        /// Final output token account.
        #[arg(long, requires = "final_output_token")]
        final_output_token_account: Option<Pubkey>,
        /// Secondary output token account.
        #[arg(long)]
        secondary_output_token_account: Option<Pubkey>,
        /// Swap paths for output token (collateral token).
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
    },
    /// Liquidate the given position.
    Liquidate {
        /// The address of the position to liquidate.
        position: Pubkey,
        /// Final output token.
        #[arg(long, short = 'o')]
        final_output_token: Option<Pubkey>,
        /// Final output token account.
        #[arg(long, requires = "final_output_token")]
        final_output_token_account: Option<Pubkey>,
        /// Secondary output token account.
        #[arg(long)]
        secondary_output_token_account: Option<Pubkey>,
        /// Swap paths for output token (collateral token).
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum Side {
    Long,
    Short,
}

impl Side {
    fn is_long(&self) -> bool {
        matches!(self, Self::Long)
    }
}

impl ExchangeArgs {
    pub(super) async fn run(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<()> {
        match &self.command {
            Command::CreateDeposit {
                extra_execution_fee,
                market_token,
                receiver,
                min_amount,
                long_token,
                short_token,
                long_token_account,
                short_token_account,
                long_token_amount,
                short_token_amount,
                long_swap,
                short_swap,
            } => {
                let mut builder = client.create_deposit(store, market_token);
                if let Some(receiver) = receiver {
                    builder.receiver(receiver);
                }
                if *long_token_amount != 0 {
                    builder.long_token(
                        *long_token_amount,
                        long_token.as_ref(),
                        long_token_account.as_ref(),
                    );
                }
                if *short_token_amount != 0 {
                    builder.short_token(
                        *short_token_amount,
                        short_token.as_ref(),
                        short_token_account.as_ref(),
                    );
                }
                let (builder, deposit) = builder
                    .execution_fee(*extra_execution_fee)
                    .min_market_token(*min_amount)
                    .long_token_swap_path(long_swap.clone())
                    .short_token_swap_path(short_swap.clone())
                    .build_with_address()
                    .await?;
                let signature = builder.send().await?;
                println!("created deposit {deposit} at {signature}");
            }
            Command::CancelDeposit { deposit } => {
                let signature = client
                    .cancel_deposit(store, deposit)
                    .build()
                    .await?
                    .send()
                    .await?;
                tracing::info!(%deposit, "cancelled deposit at tx {signature}");
                println!("{signature}");
            }
            Command::CreateWithdrawal {
                market_token,
                extra_execution_fee,
                amount,
                long_token,
                short_token,
                market_token_account,
                long_token_account,
                short_token_account,
                min_long_token_amount,
                min_short_token_amount,
                long_swap,
                short_swap,
            } => {
                let mut builder = client.create_withdrawal(store, market_token, *amount);
                if let Some(account) = market_token_account {
                    builder.market_token_account(account);
                }
                if let Some(token) = long_token {
                    builder.final_long_token(token, long_token_account.as_ref());
                }
                if let Some(token) = short_token {
                    builder.final_short_token(token, short_token_account.as_ref());
                }
                let (builder, withdrawal) = builder
                    .execution_fee(*extra_execution_fee)
                    .min_final_long_token_amount(*min_long_token_amount)
                    .min_final_short_token_amount(*min_short_token_amount)
                    .long_token_swap_path(long_swap.clone())
                    .short_token_swap_path(short_swap.clone())
                    .build_with_address()
                    .await?;
                let signature = builder.send().await?;
                println!("created withdrawal {withdrawal} at {signature}");
            }
            Command::CancelWithdrawal { withdrawal } => {
                let signature = client
                    .cancel_withdrawal(store, withdrawal)
                    .build()
                    .await?
                    .send()
                    .await?;
                tracing::info!(%withdrawal, "cancelled withdrawal at tx {signature}");
                println!("{signature}");
            }
            Command::MarketIncrease {
                market_token,
                collateral_side,
                initial_collateral_token,
                initial_collateral_token_account,
                initial_collateral_token_amount,
                side,
                size,
                swap,
            } => {
                let mut builder = client.market_increase(
                    store,
                    market_token,
                    collateral_side.is_long(),
                    *initial_collateral_token_amount,
                    side.is_long(),
                    *size,
                );
                if let Some(token) = initial_collateral_token {
                    builder
                        .initial_collateral_token(token, initial_collateral_token_account.as_ref());
                }

                let (request, order) = builder.swap_path(swap.clone()).build_with_address().await?;
                let signature = request.send().await?;
                println!("created market increase order {order} at tx {signature}");
            }
            Command::MarketDecrease {
                market_token,
                collateral_side,
                collateral_withdrawal_amount,
                side,
                size,
                final_output_token,
                final_output_token_account,
                secondary_output_token_account,
                swap,
            } => {
                let mut builder = client.market_decrease(
                    store,
                    market_token,
                    collateral_side.is_long(),
                    *collateral_withdrawal_amount,
                    side.is_long(),
                    *size,
                );
                if let Some(token) = final_output_token {
                    builder.final_output_token(token, final_output_token_account.as_ref());
                }
                if let Some(account) = secondary_output_token_account {
                    builder.secondary_output_token_account(account);
                }
                let (request, order) = builder.swap_path(swap.clone()).build_with_address().await?;
                let signature = request.send().await?;
                println!("created market decrease order {order} at tx {signature}");
            }
            Command::Liquidate {
                position,
                final_output_token,
                final_output_token_account,
                secondary_output_token_account,
                swap,
            } => {
                let position = utils::try_deserailize_account::<Position>(
                    &client.data_store().async_rpc(),
                    position,
                )
                .await?;
                let market = client.find_market_address(store, &position.market_token);
                let market = client.data_store().account::<Market>(market).await?;
                let is_collateral_token_long =
                    if market.meta().long_token_mint == position.collateral_token {
                        true
                    } else if market.meta().short_token_mint == position.collateral_token {
                        false
                    } else {
                        return Err(gmsol::Error::invalid_argument(
                            "the collateral token is not valid in the market",
                        ));
                    };
                let mut builder = client.liquidate(
                    store,
                    &position.market_token,
                    is_collateral_token_long,
                    position
                        .is_long()
                        .map_err(anchor_client::ClientError::from)?,
                    Some(position.size_in_usd),
                );
                if let Some(token) = final_output_token {
                    builder.final_output_token(token, final_output_token_account.as_ref());
                }
                if let Some(account) = secondary_output_token_account {
                    builder.secondary_output_token_account(account);
                }
                let (request, order) = builder.swap_path(swap.clone()).build_with_address().await?;
                let signature = request.send().await?;
                println!("created liquidation order {order} at tx {signature}");
            }
        }
        Ok(())
    }
}
