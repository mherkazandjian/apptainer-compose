use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct KillArgs {
    /// Services to kill
    pub services: Vec<String>,

    /// SIGNAL to send to the container
    #[arg(short = 's', long, default_value = "SIGKILL")]
    pub signal: String,

    /// Remove containers for services not defined in the Compose file
    #[arg(long)]
    pub remove_orphans: bool,
}

pub async fn run(global: GlobalOpts, args: KillArgs) -> Result<()> {
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
            tracing::info!("[dry-run] Would kill {service_name} with {}", args.signal);
            continue;
        }

        match apptainer
            .instance_stop(&instance_name, Some(&args.signal), Some(0))
            .await
        {
            Ok(()) => {
                tracing::info!("Killed service: {service_name}");
                state.set_instance_stopped(service_name);
            }
            Err(e) => tracing::warn!("Failed to kill {service_name}: {e}"),
        }
    }

    state.save(&project_dir)?;
    Ok(())
}
