use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::planner::dependency;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct DownArgs {
    /// Services to stop
    pub services: Vec<String>,

    /// Remove named volumes declared in the "volumes" section of the Compose file
    #[arg(long)]
    pub volumes: bool,

    /// Remove images used by services
    #[arg(long)]
    pub rmi: Option<String>,

    /// Remove containers for services not defined in the Compose file
    #[arg(long)]
    pub remove_orphans: bool,

    /// Specify a shutdown timeout in seconds
    #[arg(short = 't', long)]
    pub timeout: Option<u32>,
}

pub async fn run(global: GlobalOpts, args: DownArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let mut state = ProjectState::load_or_default(&project_dir, &project_name);

    // Resolve service order (reversed for shutdown)
    let target_services = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };
    let ordered = dependency::resolve_order(&compose.services, &target_services)?;

    // Stop in reverse dependency order
    for service_name in ordered.iter().rev() {
        let instance_name = format!("{}_{}_1", project_name, service_name);

        if global.dry_run {
            tracing::info!("[dry-run] Would stop service: {service_name}");
            continue;
        }

        tracing::info!("Stopping service: {service_name}");
        match apptainer
            .instance_stop(
                &instance_name,
                None,
                args.timeout.map(|t| t as i32),
            )
            .await
        {
            Ok(()) => {
                tracing::info!("Stopped service: {service_name}");
                state.set_instance_stopped(service_name);
            }
            Err(e) => {
                tracing::warn!("Failed to stop {service_name}: {e}");
            }
        }
    }

    // Remove volumes if requested
    if args.volumes {
        crate::driver::volume::remove_volumes(&project_dir)?;
        tracing::info!("Removed volumes");
    }

    // Remove images if requested
    if let Some(rmi_mode) = &args.rmi {
        if rmi_mode == "all" || rmi_mode == "local" {
            crate::driver::image::remove_images(&project_dir)?;
            tracing::info!("Removed images");
        }
    }

    // Remove hosts files
    crate::driver::network::cleanup_hosts_files(&project_dir)?;

    state.save(&project_dir)?;

    Ok(())
}
