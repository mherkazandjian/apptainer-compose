use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::driver::volume;
use crate::error::Result;

#[derive(Args)]
pub struct CreateArgs {
    /// Services to create
    pub services: Vec<String>,

    /// Build images before starting containers
    #[arg(long)]
    pub build: bool,

    /// Recreate containers even if their configuration and image haven't changed
    #[arg(long)]
    pub force_recreate: bool,

    /// If containers already exist, don't recreate them
    #[arg(long)]
    pub no_recreate: bool,

    /// Don't build an image, even if it's policy
    #[arg(long)]
    pub no_build: bool,

    /// Pull image before running
    #[arg(long, default_value = "policy")]
    pub pull: String,

    /// Pull without printing progress information
    #[arg(long)]
    pub quiet_pull: bool,

    /// Remove containers for services not defined in the Compose file
    #[arg(long)]
    pub remove_orphans: bool,

    /// Scale SERVICE to NUM instances
    #[arg(long)]
    pub scale: Vec<String>,

    /// Assume "yes" as answer to all prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub async fn run(global: GlobalOpts, args: CreateArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;

    let services: Vec<String> = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let service = &compose.services[service_name];
        tracing::info!("Creating service: {service_name}");

        if global.dry_run {
            tracing::info!("[dry-run] Would create service: {service_name}");
            continue;
        }

        // Pull/build image
        image::ensure_image(&apptainer, &project_dir, service_name, service).await?;

        // Create volumes
        volume::ensure_volumes(&project_dir, &compose, service_name, &apptainer.binary)?;
    }

    Ok(())
}
