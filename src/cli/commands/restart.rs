use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::driver::instance;
use crate::driver::network;
use crate::error::Result;
use crate::planner::reconciler;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct RestartArgs {
    /// Services to restart
    pub services: Vec<String>,

    /// Specify a shutdown timeout in seconds
    #[arg(short = 't', long)]
    pub timeout: Option<u32>,

    /// Don't restart dependent services
    #[arg(long)]
    pub no_deps: bool,
}

pub async fn run(global: GlobalOpts, args: RestartArgs) -> Result<()> {
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
        let service = &compose.services[service_name];

        if global.dry_run {
            tracing::info!("[dry-run] Would restart service: {service_name}");
            continue;
        }

        // Stop
        let _ = apptainer
            .instance_stop(&instance_name, None, args.timeout.map(|t| t as i32))
            .await;

        // Start
        let image_path = image::ensure_image(&apptainer, &project_dir, service_name, service).await?;
        let hosts_file = network::generate_hosts_file(&project_dir, &compose.services)?;

        let start_args = reconciler::build_start_args(
            &project_dir,
            &project_name,
            service_name,
            service,
            &compose,
            &image_path,
            &hosts_file,
        )?;

        instance::start_instance(&apptainer, &start_args).await?;
        tracing::info!("Restarted service: {service_name}");

        state.set_instance_running(
            service_name,
            &instance_name,
            &image_path,
            service.image.as_deref().unwrap_or(""),
        );
    }

    state.save(&project_dir)?;
    Ok(())
}
