use clap::Parser;
use skills_store_cli::commands::strategy_grid::{execute, GridCommand};
use skills_store_cli::output;

#[derive(Parser)]
#[command(
    name = "strategy-grid",
    version,
    about = "ETH/USDC grid trading bot on Base"
)]
struct Cli {
    #[command(subcommand)]
    command: GridCommand,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute(cli.command).await {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
