use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::error::Result;

#[derive(Args)]
pub struct PullArgs {
    /// Services to pull images for
    pub services: Vec<String>,

    /// Pull without printing progress information
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Ignore images that can be built
    #[arg(long)]
    pub ignore_buildable: bool,

    /// Pull what it can and ignores images with pull failures
    #[arg(long)]
    pub ignore_pull_failures: bool,

    /// Also pull services declared as dependencies
    #[arg(long)]
    pub include_deps: bool,

    /// Apply pull policy
    #[arg(long)]
    pub policy: Option<String>,
}

pub async fn run(global: GlobalOpts, args: PullArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;

    let services: Vec<String> = if args.services.is_empty() {
        compose
            .services
            .iter()
            .filter(|(_, s)| s.image.is_some())
            .map(|(k, _)| k.clone())
            .collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let service = &compose.services[service_name];
        if let Some(ref img) = service.image {
            tracing::info!("Pulling image for {service_name}: {img}");
            if global.dry_run {
                tracing::info!("[dry-run] Would pull: {img}");
                continue;
            }
            image::pull_image(&apptainer, &project_dir, service_name, img).await?;
        }
    }

    Ok(())
}
