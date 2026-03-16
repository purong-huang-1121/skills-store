use clap::Parser;
use skills_store_cli::commands::strategy_memepump_scanner::{execute, ScannerCommand};
use skills_store_cli::output;

#[derive(Parser)]
#[command(
    name = "strategy-memepump-scanner",
    version,
    about = "SOL memepump scanner — automated pump.fun token trading"
)]
struct Cli {
    #[command(subcommand)]
    command: ScannerCommand,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute(cli.command).await {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
