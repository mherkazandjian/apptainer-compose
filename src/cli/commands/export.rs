use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct ExportArgs {
    /// Service name
    pub service: String,

    /// Write to a file, instead of STDOUT
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,
}

pub async fn run(_global: GlobalOpts, _args: ExportArgs) -> Result<()> {
    tracing::warn!("export command is not yet implemented for Apptainer");
    tracing::info!("Apptainer images are already portable SIF files");
    Ok(())
}
