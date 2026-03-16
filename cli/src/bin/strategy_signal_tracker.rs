use clap::Parser;
use skills_store_cli::commands::strategy_signal_tracker::{execute, SignalTrackerCommand};
use skills_store_cli::output;

#[derive(Parser)]
#[command(
    name = "strategy-signal-tracker",
    version,
    about = "SOL signal tracker — follow smart money signals with safety filter"
)]
struct Cli {
    #[command(subcommand)]
    command: SignalTrackerCommand,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute(cli.command).await {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
