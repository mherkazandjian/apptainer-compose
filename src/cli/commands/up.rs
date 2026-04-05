use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::driver::instance;
use crate::driver::logs as driver_logs;
use crate::driver::network;
use crate::driver::volume;
use crate::error::Result;
use crate::planner::dependency;
use crate::planner::reconciler;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct UpArgs {
    /// Services to start
    pub services: Vec<String>,

    /// Detached mode: Run containers in the background
    #[arg(short = 'd', long)]
    pub detach: bool,

    /// Build images before starting containers
    #[arg(long)]
    pub build: bool,

    /// Don't build an image, even if it's policy
    #[arg(long)]
    pub no_build: bool,

    /// Recreate containers even if their configuration and image haven't changed
    #[arg(long)]
    pub force_recreate: bool,

    /// If containers already exist, don't recreate them
    #[arg(long)]
    pub no_recreate: bool,

    /// Don't start linked services
    #[arg(long)]
    pub no_deps: bool,

    /// Recreate dependent containers
    #[arg(long)]
    pub always_recreate_deps: bool,

    /// Recreate anonymous volumes instead of retrieving data from the previous containers
    #[arg(short = 'V', long)]
    pub renew_anon_volumes: bool,

    /// Remove containers for services not defined in the Compose file
    #[arg(long)]
    pub remove_orphans: bool,

    /// Pull image before running
    #[arg(long, default_value = "policy")]
    pub pull: String,

    /// Don't start the services after creating them
    #[arg(long)]
    pub no_start: bool,

    /// Produce monochrome output
    #[arg(long)]
    pub no_color: bool,

    /// Don't print prefix in logs
    #[arg(long)]
    pub no_log_prefix: bool,

    /// Show timestamps
    #[arg(long)]
    pub timestamps: bool,

    /// Use this timeout in seconds for container shutdown
    #[arg(short = 't', long)]
    pub timeout: Option<u32>,

    /// Wait for services to be running|healthy
    #[arg(long)]
    pub wait: bool,

    /// Maximum duration in seconds to wait for the project to be running|healthy
    #[arg(long)]
    pub wait_timeout: Option<u32>,

    /// Scale SERVICE to NUM instances
    #[arg(long)]
    pub scale: Vec<String>,

    /// Restrict attaching to the specified services
    #[arg(long)]
    pub attach: Vec<String>,

    /// Do not attach (stream logs) to the specified services
    #[arg(long)]
    pub no_attach: Vec<String>,

    /// Suppress the build output
    #[arg(long)]
    pub quiet_build: bool,

    /// Pull without printing progress information
    #[arg(long)]
    pub quiet_pull: bool,

    /// Assume "yes" as answer to all prompts
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Watch source code and rebuild/refresh containers when files are updated
    #[arg(short = 'w', long)]
    pub watch: bool,

    /// Automatically attach to log output of dependent services
    #[arg(long)]
    pub attach_dependencies: bool,

    /// Stops all containers if any container was stopped
    #[arg(long)]
    pub abort_on_container_exit: bool,

    /// Stops all containers if any container exited with failure
    #[arg(long)]
    pub abort_on_container_failure: bool,

    /// Return the exit code of the selected service container
    #[arg(long)]
    pub exit_code_from: Option<String>,
}

pub async fn run(global: GlobalOpts, args: UpArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let mut state = ProjectState::load_or_default(&project_dir, &project_name);

    // Reconcile with live apptainer state
    let live_instances = apptainer.instance_list().await?;
    state.reconcile_with_live(&live_instances);

    // Resolve service order
    let target_services = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };
    let ordered = dependency::resolve_order(&compose.services, &target_services)?;

    // Process each service in dependency order
    for service_name in &ordered {
        let service = &compose.services[service_name];
        tracing::info!("Processing service: {service_name}");

        if global.dry_run {
            tracing::info!("[dry-run] Would process service: {service_name}");
            continue;
        }

        // Pull/build image
        let image_path = image::ensure_image(
            &apptainer,
            &project_dir,
            service_name,
            service,
        )
        .await?;

        // Create volumes
        volume::ensure_volumes(&project_dir, &compose, service_name)?;

        // Generate /etc/hosts for service discovery
        let hosts_file = network::generate_hosts_file(
            &project_dir,
            &compose.services,
        )?;

        // Determine instance name
        let instance_name = format!("{}_{}_1", project_name, service_name);

        // Check if already running
        if state.is_instance_running(&instance_name) && !args.force_recreate {
            tracing::info!("Service {service_name} is already running");
            continue;
        }

        // Stop existing if force recreate
        if args.force_recreate && state.is_instance_running(&instance_name) {
            apptainer.instance_stop(&instance_name, None, None).await?;
        }

        // Build start args
        let start_args = reconciler::build_start_args(
            &project_dir,
            &project_name,
            service_name,
            service,
            &compose,
            &image_path,
            &hosts_file,
        )?;

        // Start instance
        if !args.no_start {
            instance::start_instance(&apptainer, &start_args).await?;
            tracing::info!("Started service: {service_name}");

            state.set_instance_running(
                service_name,
                &instance_name,
                &image_path,
                service.image.as_deref().unwrap_or(""),
            );
        }
    }

    state.save(&project_dir)?;

    // Stream logs if not detached
    if !args.detach && !args.no_start {
        let instance_names: Vec<String> = ordered
            .iter()
            .map(|s| format!("{}_{}_1", project_name, s))
            .collect();
        driver_logs::stream_logs(&apptainer, &instance_names, true, args.no_log_prefix, args.timestamps).await?;
    }

    Ok(())
}
