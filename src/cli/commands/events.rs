use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct EventsArgs {
    /// Services to show events for
    pub services: Vec<String>,

    /// Output events as a stream of json objects
    #[arg(long)]
    pub json: bool,

    /// Show all events created since timestamp
    #[arg(long)]
    pub since: Option<String>,

    /// Stream events until this timestamp
    #[arg(long)]
    pub until: Option<String>,
}

pub async fn run(_global: GlobalOpts, _args: EventsArgs) -> Result<()> {
    tracing::warn!("events command is not yet implemented for Apptainer");
    tracing::info!("Apptainer does not have a native event stream; consider using 'ps' for status");
    Ok(())
}
