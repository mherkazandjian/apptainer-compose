use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct CommitArgs {
    /// Service name
    pub service: String,

    /// Repository and optional tag
    pub repository: Option<String>,

    /// Author
    #[arg(short = 'a', long)]
    pub author: Option<String>,

    /// Apply Dockerfile instruction to the created image
    #[arg(short = 'c', long)]
    pub change: Vec<String>,

    /// Commit message
    #[arg(short = 'm', long)]
    pub message: Option<String>,

    /// Pause container during commit
    #[arg(short = 'p', long, default_value_t = true)]
    pub pause: bool,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,
}

pub async fn run(_global: GlobalOpts, _args: CommitArgs) -> Result<()> {
    tracing::warn!("commit command is not yet implemented for Apptainer");
    tracing::info!("Apptainer SIF images are immutable; use sandbox mode and 'apptainer build' to create new images");
    Ok(())
}
