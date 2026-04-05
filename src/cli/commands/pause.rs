use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct PauseArgs {
    /// Services to pause
    pub services: Vec<String>,
}

pub async fn run(global: GlobalOpts, args: PauseArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let services: Vec<String> = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let instance_name = format!("{}_{}_1", project_name, service_name);

        if global.dry_run {
            tracing::info!("[dry-run] Would pause service: {service_name}");
            continue;
        }

        // Apptainer has no native pause; use SIGSTOP via instance PID
        apptainer.signal_instance(&instance_name, "SIGSTOP").await?;
        tracing::info!("Paused service: {service_name}");
    }

    Ok(())
}
