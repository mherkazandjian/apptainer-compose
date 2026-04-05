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
    let cli = Cli::parse();

    logging::init(&cli.global);

    if let Err(e) = cli::dispatch(cli).await {
        tracing::error!("{e:#}");
        std::process::exit(1);
    }
}
