use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct WatchArgs {
    /// Services to watch
    pub services: Vec<String>,

    /// Do not build & start services before watching
    #[arg(long)]
    pub no_up: bool,

    /// Prune dangling images on rebuild
    #[arg(long, default_value_t = true)]
    pub prune: bool,

    /// Hide build output
    #[arg(long)]
    pub quiet: bool,
}

pub async fn run(_global: GlobalOpts, _args: WatchArgs) -> Result<()> {
    tracing::warn!("watch command is not yet implemented for Apptainer");
    tracing::info!("This feature requires file system watching and automatic rebuild/restart");
    Ok(())
}
