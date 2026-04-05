use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::image;
use crate::error::Result;
use crate::planner::reconciler;

#[derive(Args)]
pub struct RunArgs {
    /// Service name
    pub service: String,

    /// Command and arguments
    pub command: Vec<String>,

    /// Run container in background and print container ID
    #[arg(short = 'd', long)]
    pub detach: bool,

    /// Set environment variables
    #[arg(short = 'e', long)]
    pub env: Vec<String>,

    /// Set environment variables from file
    #[arg(long)]
    pub env_from_file: Vec<String>,

    /// Override the entrypoint of the image
    #[arg(long)]
    pub entrypoint: Option<String>,

    /// Assign a name to the container
    #[arg(long)]
    pub name: Option<String>,

    /// Don't start linked services
    #[arg(long)]
    pub no_deps: bool,

    /// Automatically remove the container when it exits
    #[arg(long)]
    pub rm: bool,

    /// Run command with all service's ports enabled
    #[arg(short = 'P', long)]
    pub service_ports: bool,

    /// Run as specified username or uid
    #[arg(short = 'u', long)]
    pub user: Option<String>,

    /// Bind mount a volume
    #[arg(short = 'v', long)]
    pub volume: Vec<String>,

    /// Working directory inside the container
    #[arg(short = 'w', long)]
    pub workdir: Option<String>,

    /// Build image before starting container
    #[arg(long)]
    pub build: bool,

    /// Pull image before running
    #[arg(long, default_value = "policy")]
    pub pull: String,

    /// Disable pseudo-TTY allocation
    #[arg(short = 'T', long = "no-TTY")]
    pub no_tty: bool,

    /// Keep STDIN open even if not attached
    #[arg(short = 'i', long, default_value_t = true)]
    pub interactive: bool,

    /// Don't print anything to STDOUT
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Suppress the build output
    #[arg(long)]
    pub quiet_build: bool,

    /// Pull without printing progress information
    #[arg(long)]
    pub quiet_pull: bool,

    /// Remove containers for services not defined in the Compose file
    #[arg(long)]
    pub remove_orphans: bool,

    /// Publish a container's port(s) to the host
    #[arg(short = 'p', long)]
    pub publish: Vec<String>,

    /// Add or override a label
    #[arg(short = 'l', long)]
    pub label: Vec<String>,

    /// Add Linux capabilities
    #[arg(long)]
    pub cap_add: Vec<String>,

    /// Drop Linux capabilities
    #[arg(long)]
    pub cap_drop: Vec<String>,

    /// Use the service's network useAliases
    #[arg(long)]
    pub use_aliases: bool,
}

pub async fn run(global: GlobalOpts, args: RunArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let service = compose
        .services
        .get(&args.service)
        .ok_or_else(|| crate::error::AppError::Other(format!("service '{}' not found", args.service)))?;

    let image_path = image::ensure_image(&apptainer, &project_dir, &args.service, service).await?;

    if global.dry_run {
        tracing::info!("[dry-run] Would run one-off command in {}: {:?}", args.service, args.command);
        return Ok(());
    }

    let hosts_file = crate::driver::network::generate_hosts_file(&project_dir, &compose.services)?;

    let start_args = reconciler::build_start_args(
        &project_dir,
        &project_name,
        &args.service,
        service,
        &compose,
        &image_path,
        &hosts_file,
    )?;

    let status = apptainer
        .run_oneoff(&start_args, &args.command)
        .await?;

    std::process::exit(status);
}
