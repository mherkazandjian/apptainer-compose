use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct StopArgs {
    /// Services to stop
    pub services: Vec<String>,

    /// Specify a shutdown timeout in seconds
    #[arg(short = 't', long)]
    pub timeout: Option<u32>,
}

pub async fn run(global: GlobalOpts, args: StopArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let mut state = ProjectState::load_or_default(&project_dir, &project_name);

    let services: Vec<String> = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let instance_name = format!("{}_{}_1", project_name, service_name);

        if global.dry_run {
            tracing::info!("[dry-run] Would stop service: {service_name}");
            continue;
        }

        match apptainer
            .instance_stop(&instance_name, None, args.timeout.map(|t| t as i32))
            .await
        {
            Ok(()) => {
                tracing::info!("Stopped service: {service_name}");
                state.set_instance_stopped(service_name);
            }
            Err(e) => tracing::warn!("Failed to stop {service_name}: {e}"),
        }
    }

    state.save(&project_dir)?;
    Ok(())
}
