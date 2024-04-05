use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::exchange::ExchangeOps;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct ExchangeArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Create a deposit.
    CreateDeposit {
        /// The address of the market token of the Market to deposit into.
        #[arg(long, short)]
        market_token: Pubkey,
        /// The token account to receive the minted market tokens.
        ///
        /// Defaults to use assciated token account.
        #[arg(long, short)]
        receiver: Option<Pubkey>,
        /// The initial long token account.
        #[arg(long)]
        long_token_account: Option<Pubkey>,
        /// The initial short token account.
        #[arg(long)]
        short_token_account: Option<Pubkey>,
        /// The initial long token amount.
        #[arg(long, default_value_t = 0, requires = "long_token_account")]
        long_token_amount: u64,
        /// The initial short token amount.
        #[arg(long, default_value_t = 0, requires = "short_token_account")]
        short_token_amount: u64,
    },
}

impl ExchangeArgs {
    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> eyre::Result<()> {
        let program = client.program(exchange::id())?;
        match &self.command {
            Command::CreateDeposit {
                market_token,
                receiver,
                long_token_account,
                short_token_account,
                long_token_amount,
                short_token_amount,
            } => {
                let mut builder = program.create_deposit(store, market_token);
                if let Some(receiver) = receiver {
                    builder.receiver(receiver);
                }
                if let Some(token_account) = long_token_account {
                    builder
                        .long_token(token_account, *long_token_amount, None)
                        .await?;
                }
                if let Some(token_account) = short_token_account {
                    builder
                        .short_token(token_account, *short_token_amount, None)
                        .await?;
                }
                let (builder, deposit) = builder.build_with_address()?;
                let signature = builder.send().await?;
                tracing::info!(%deposit, "created a deposit at tx {signature}");
                println!("{deposit}");
            }
        }
        Ok(())
    }
}
