use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct PushArgs {
    /// Services to push
    pub services: Vec<String>,

    /// Push without printing progress information
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Push what it can and ignores images with push failures
    #[arg(long)]
    pub ignore_push_failures: bool,

    /// Also push images of services declared as dependencies
    #[arg(long)]
    pub include_deps: bool,
}

pub async fn run(_global: GlobalOpts, _args: PushArgs) -> Result<()> {
    tracing::warn!("push command is not yet fully implemented for Apptainer");
    tracing::info!("Use 'apptainer push' directly for pushing SIF images to registries");
    Ok(())
}
