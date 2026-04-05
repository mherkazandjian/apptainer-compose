mod cli;
mod compose;
mod driver;
mod error;
mod logging;
mod planner;
mod state;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = parse_cli();

    logging::init(&cli.global);

    if let Err(e) = cli::dispatch(cli).await {
        tracing::error!("{e:#}");
        std::process::exit(1);
    }
}

/// Parse CLI arguments, supporting both `apptainer-compose <cmd>` and
/// `apptainer compose <cmd>` invocation styles.
fn parse_cli() -> Cli {
    let args: Vec<String> = std::env::args().collect();
    let bin_name = args
        .first()
        .and_then(|a| std::path::Path::new(a).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // When invoked as "apptainer" (not "apptainer-compose"), expect "compose"
    // as the first argument and strip it, so "apptainer compose up" becomes
    // equivalent to "apptainer-compose up".
    if bin_name == "apptainer" {
        if args.len() > 1 && args[1] == "compose" {
            let filtered: Vec<String> = std::iter::once(args[0].clone())
                .chain(args[2..].iter().cloned())
                .collect();
            Cli::parse_from(filtered)
        } else {
            eprintln!("Usage: apptainer compose <COMMAND> [OPTIONS]");
            eprintln!("Run 'apptainer compose --help' for more information.");
            std::process::exit(1);
        }
    } else {
        Cli::parse()
    }
}
