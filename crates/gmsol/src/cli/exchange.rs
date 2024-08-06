use anchor_client::solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use eyre::OptionExt;
use gmsol::{exchange::ExchangeOps, utils::price_to_min_output_amount};
use rust_decimal::Decimal;

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
    /// Cancel an order.
    CancelOrder {
        /// The address of the order to cancel.
        order: Pubkey,
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
        /// Whether to wait for the action to be completed.
        #[arg(long, short)]
        wait: bool,
    },
    /// Create a limit increase order.
    LimitIncrease {
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
        /// Trigger price (unit price).
        #[arg(long)]
        price: u128,
        /// Position increment size in usd.
        #[arg(long)]
        size: u128,
        /// Swap paths for collateral token.
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
        /// Whether to wait for the action to be completed.
        #[arg(long, short)]
        wait: bool,
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
        /// Whether to wait for the action to be completed.
        #[arg(long, short)]
        wait: bool,
    },
    /// Create a limit decrese order.
    LimitDecrease {
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
        /// Trigger price (unit price).
        #[arg(long)]
        price: u128,
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
        /// Whether to wait for the action to be completed.
        #[arg(long, short)]
        wait: bool,
    },
    /// Create a market swap order.
    MarketSwap {
        /// The address of the market token of the position's market.
        market_token: Pubkey,
        /// Output side.
        #[arg(long)]
        output_side: Side,
        /// Initial swap in token.
        #[arg(long, short = 'i')]
        initial_swap_in_token: Pubkey,
        /// Initial swap in token account.
        #[arg(long)]
        initial_swap_in_token_account: Option<Pubkey>,
        /// Collateral amount.
        #[arg(long, short = 'a')]
        initial_swap_in_token_amount: u64,
        /// Extra swap path. No need to provide the target market token;
        /// it will be automatically added to the end of the swap path.
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
    },
    /// Create a limit swap order.
    LimitSwap {
        /// The address of the market token of the position's market.
        market_token: Pubkey,
        /// Output side.
        #[arg(long)]
        output_side: Side,
        /// Limit price (`token_in` to `token_out` price)
        #[arg(long, value_parser = parse_decimal)]
        price: Decimal,
        /// Initial swap in token.
        #[arg(long, short = 'i')]
        initial_swap_in_token: Pubkey,
        /// Initial swap in token account.
        #[arg(long)]
        initial_swap_in_token_account: Option<Pubkey>,
        /// Collateral amount.
        #[arg(long, short = 'a')]
        initial_swap_in_token_amount: u64,
        /// Extra swap path. No need to provide the target market token;
        /// it will be automatically added to the end of the swap path.
        #[arg(long, short, action = clap::ArgAction::Append)]
        swap: Vec<Pubkey>,
    },
}

fn parse_decimal(value: &str) -> Result<Decimal, clap::Error> {
    value
        .parse::<Decimal>()
        .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidValue))
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
            Command::CancelOrder { order } => {
                let signature = client
                    .cancel_order(order)?
                    .build()
                    .await?
                    .build()
                    .send()
                    .await?;
                tracing::info!(%order, "cancelled order at tx {signature}");
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
                wait,
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
                tracing::info!("created a market increase order {order} at tx {signature}");
                if *wait {
                    self.wait_for_order(client, &order).await?;
                }
                println!("{order}");
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
                wait,
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
                tracing::info!("created a market decrease order {order} at tx {signature}");
                if *wait {
                    self.wait_for_order(client, &order).await?;
                }
                println!("{order}");
            }
            Command::MarketSwap {
                market_token,
                output_side,
                initial_swap_in_token,
                initial_swap_in_token_account,
                initial_swap_in_token_amount,
                swap,
            } => {
                let mut builder = client.market_swap(
                    store,
                    market_token,
                    output_side.is_long(),
                    initial_swap_in_token,
                    *initial_swap_in_token_amount,
                    swap.iter().chain(Some(market_token)),
                );
                if let Some(account) = initial_swap_in_token_account {
                    builder.initial_collateral_token(initial_swap_in_token, Some(account));
                }

                let (request, order) = builder.build_with_address().await?;
                let signature = request.send().await?;
                tracing::info!("created a market swap order {order} at tx {signature}");
                println!("{order}");
            }
            Command::LimitIncrease {
                market_token,
                collateral_side,
                initial_collateral_token,
                initial_collateral_token_account,
                initial_collateral_token_amount,
                side,
                price,
                size,
                swap,
                wait,
            } => {
                let mut builder = client.limit_increase(
                    store,
                    market_token,
                    side.is_long(),
                    *size,
                    *price,
                    collateral_side.is_long(),
                    *initial_collateral_token_amount,
                );
                if let Some(token) = initial_collateral_token {
                    builder
                        .initial_collateral_token(token, initial_collateral_token_account.as_ref());
                }

                let (request, order) = builder.swap_path(swap.clone()).build_with_address().await?;
                let signature = request.send().await?;
                tracing::info!("created a limit increase order {order} at tx {signature}");
                if *wait {
                    self.wait_for_order(client, &order).await?;
                }
                println!("{order}");
            }
            Command::LimitDecrease {
                market_token,
                collateral_side,
                collateral_withdrawal_amount,
                side,
                price,
                size,
                final_output_token,
                final_output_token_account,
                secondary_output_token_account,
                swap,
                wait,
            } => {
                let mut builder = client.limit_decrease(
                    store,
                    market_token,
                    side.is_long(),
                    *size,
                    *price,
                    collateral_side.is_long(),
                    *collateral_withdrawal_amount,
                );
                if let Some(token) = final_output_token {
                    builder.final_output_token(token, final_output_token_account.as_ref());
                }
                if let Some(account) = secondary_output_token_account {
                    builder.secondary_output_token_account(account);
                }
                let (request, order) = builder.swap_path(swap.clone()).build_with_address().await?;
                let signature = request.send().await?;
                tracing::info!("created a limit decrease order {order} at tx {signature}");
                if *wait {
                    self.wait_for_order(client, &order).await?;
                }
                println!("{order}");
            }
            Command::LimitSwap {
                market_token,
                output_side,
                price,
                initial_swap_in_token,
                initial_swap_in_token_account,
                initial_swap_in_token_amount,
                swap,
            } => {
                let token_map = client
                    .token_map(
                        &client
                            .authorized_token_map(store)
                            .await?
                            .ok_or_eyre("token map is not set")?,
                    )
                    .await?;
                let market = client
                    .market(&client.find_market_address(store, market_token))
                    .await?;
                let token_out = market.meta().pnl_token(output_side.is_long());
                let min_output_amount = price_to_min_output_amount(
                    &token_map,
                    initial_swap_in_token,
                    *initial_swap_in_token_amount,
                    &token_out,
                    *price,
                )
                .ok_or_eyre("invalid price")?;
                let mut builder = client.limit_swap(
                    store,
                    market_token,
                    output_side.is_long(),
                    min_output_amount,
                    initial_swap_in_token,
                    *initial_swap_in_token_amount,
                    swap.iter().chain(Some(market_token)),
                );
                if let Some(account) = initial_swap_in_token_account {
                    builder.initial_collateral_token(initial_swap_in_token, Some(account));
                }

                let (request, order) = builder.build_with_address().await?;
                let signature = request.send().await?;
                tracing::info!("created a market swap order {order} at tx {signature}");
                println!("{order}");
            }
        }
        Ok(())
    }

    async fn wait_for_order(&self, client: &GMSOLClient, order: &Pubkey) -> gmsol::Result<()> {
        let trade = client
            .complete_order(order, Some(CommitmentConfig::confirmed()))
            .await?;
        match trade {
            Some(trade) => {
                tracing::info!(%order, "order completed with trade event: {trade:#}");
            }
            None => {
                tracing::warn!(%order, "order completed without trade event");
            }
        }
        Ok(())
    }
}
