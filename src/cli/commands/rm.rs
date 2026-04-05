use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct RmArgs {
    /// Services to remove
    pub services: Vec<String>,

    /// Don't ask to confirm removal
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Stop the containers, if required, before removing
    #[arg(short = 's', long)]
    pub stop: bool,

    /// Remove any anonymous volumes attached to containers
    #[arg(short = 'v', long)]
    pub volumes: bool,
}

pub async fn run(global: GlobalOpts, args: RmArgs) -> Result<()> {
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
            tracing::info!("[dry-run] Would remove service: {service_name}");
            continue;
        }

        if args.stop {
            let _ = apptainer.instance_stop(&instance_name, None, None).await;
        }

        state.remove_service(service_name);
        tracing::info!("Removed service: {service_name}");
    }

    state.save(&project_dir)?;
    Ok(())
}
