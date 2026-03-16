use clap::{Parser, Subcommand, ValueEnum};
use skills_store_cli::{commands, dapp, output};

#[derive(Parser)]
#[command(
    name = "skills-store",
    version,
    about = "onchainOS CLI — on-chain DeFi operations"
)]
pub struct Cli {
    /// Output format
    #[arg(short, long, global = true, default_value = "json")]
    pub output: OutputFormat,

    /// Backend service URL (overrides config)
    #[arg(long, global = true)]
    pub base_url: Option<String>,

    /// Chain: ethereum, solana, base, bsc, polygon, arbitrum, sui, etc.
    #[arg(long, global = true)]
    pub chain: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Polymarket prediction markets
    Polymarket {
        #[command(subcommand)]
        command: commands::dapp_polymarket::PolymarketCommand,
    },
    /// Aave V3 lending protocol
    Aave {
        #[command(subcommand)]
        command: commands::dapp_aave::AaveCommand,
    },
    /// Hyperliquid perpetual and spot exchange
    Hyperliquid {
        #[command(subcommand)]
        command: commands::dapp_hyperliquid::HyperliquidCommand,
    },
    /// Kalshi regulated prediction markets (US)
    Kalshi {
        /// API environment: demo (default) or prod
        #[arg(long, default_value = "demo")]
        env: dapp::kalshi::auth::KalshiEnv,
        #[command(subcommand)]
        command: commands::dapp_kalshi::KalshiCommand,
    },
    /// Ethena sUSDe staking (yield-bearing stablecoin)
    Ethena {
        #[command(subcommand)]
        command: commands::dapp_ethena::EthenaCommand,
    },
    /// Morpho Protocol — permissionless lending markets and MetaMorpho vaults
    Morpho {
        #[command(subcommand)]
        command: commands::dapp_morpho::MorphoCommand,
    },
    /// Uniswap V3 on-chain swap and quote
    Uniswap {
        #[command(subcommand)]
        command: commands::dapp_uniswap::UniswapCommand,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Polymarket { command } => commands::dapp_polymarket::execute(command).await,
        Commands::Aave { command } => commands::dapp_aave::execute(command).await,
        Commands::Hyperliquid { command } => commands::dapp_hyperliquid::execute(command).await,
        Commands::Kalshi { env, command } => commands::dapp_kalshi::execute(command, env).await,
        Commands::Ethena { command } => commands::dapp_ethena::execute(command).await,
        Commands::Morpho { command } => commands::dapp_morpho::execute(command).await,
        Commands::Uniswap { command } => commands::dapp_uniswap::execute(command).await,
    };

    if let Err(e) = result {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
