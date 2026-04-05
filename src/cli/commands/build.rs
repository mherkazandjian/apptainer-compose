use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::error::Result;

#[derive(Args)]
pub struct BuildArgs {
    /// Services to build
    pub services: Vec<String>,

    /// Set build-time variables for services
    #[arg(long)]
    pub build_arg: Vec<String>,

    /// Do not use cache when building the image
    #[arg(long)]
    pub no_cache: bool,

    /// Always attempt to pull a newer version of the image
    #[arg(long)]
    pub pull: bool,

    /// Suppress the build output
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Push service images
    #[arg(long)]
    pub push: bool,

    /// Set memory limit for the build container
    #[arg(short = 'm', long)]
    pub memory: Option<String>,

    /// Set SSH authentications used when building service images
    #[arg(long)]
    pub ssh: Option<String>,

    /// Also build dependencies
    #[arg(long)]
    pub with_dependencies: bool,

    /// Set builder to use
    #[arg(long)]
    pub builder: Option<String>,

    /// Check build configuration
    #[arg(long)]
    pub check: bool,
}

pub async fn run(global: GlobalOpts, args: BuildArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;

    let services: Vec<String> = if args.services.is_empty() {
        compose
            .services
            .iter()
            .filter(|(_, s)| s.build.is_some())
            .map(|(k, _)| k.clone())
            .collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let service = &compose.services[service_name];
        tracing::info!("Building service: {service_name}");

        if global.dry_run {
            tracing::info!("[dry-run] Would build service: {service_name}");
            continue;
        }

        image::build_image(&apptainer, &project_dir, service_name, service).await?;
    }

    Ok(())
}
