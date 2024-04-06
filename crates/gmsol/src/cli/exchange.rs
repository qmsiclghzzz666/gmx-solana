use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::exchange::ExchangeOps;

use crate::SharedClient;

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
        /// The token account to receive the minted market tokens.
        ///
        /// Defaults to use assciated token account.
        #[arg(long, short)]
        receiver: Option<Pubkey>,
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
        #[arg(long, default_value_t = 0, requires = "long_token")]
        long_token_amount: u64,
        /// The initial short token amount.
        #[arg(long, default_value_t = 0, requires = "short_token")]
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
}

impl ExchangeArgs {
    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        let program = client.program(exchange::id())?;
        match &self.command {
            Command::CreateDeposit {
                market_token,
                receiver,
                long_token,
                short_token,
                long_token_account,
                short_token_account,
                long_token_amount,
                short_token_amount,
                long_swap,
                short_swap,
            } => {
                let mut builder = program.create_deposit(store, market_token);
                if let Some(receiver) = receiver {
                    builder.receiver(receiver);
                }
                if let Some(token) = long_token {
                    builder.long_token(token, *long_token_amount, long_token_account.as_ref());
                }
                if let Some(token) = short_token {
                    builder.short_token(token, *short_token_amount, short_token_account.as_ref());
                }
                let (builder, deposit) = builder
                    .long_token_swap_path(long_swap.clone())
                    .short_token_swap_path(short_swap.clone())
                    .build_with_address()?;
                let signature = builder.send().await?;
                tracing::info!(%deposit, "created deposit at tx {signature}");
                println!("{deposit}");
            }
            Command::CancelDeposit { deposit } => {
                let signature = program
                    .cancel_deposit(store, deposit)
                    .build()
                    .await?
                    .send()
                    .await?;
                tracing::info!(%deposit, "cancelled deposit at tx {signature}");
                println!("{deposit}");
            }
        }
        Ok(())
    }
}
