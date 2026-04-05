use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;

#[derive(Args)]
pub struct ScaleArgs {
    /// SERVICE=REPLICAS pairs
    pub scale: Vec<String>,

    /// Don't start linked services
    #[arg(long)]
    pub no_deps: bool,
}

pub async fn run(_global: GlobalOpts, _args: ScaleArgs) -> Result<()> {
    tracing::warn!("scale command is not yet fully implemented for Apptainer");
    tracing::info!("Hint: Use 'up --scale SERVICE=NUM' instead");
    Ok(())
}
