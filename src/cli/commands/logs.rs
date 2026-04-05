use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::driver::logs as driver_logs;
use crate::error::Result;

#[derive(Args)]
pub struct LogsArgs {
    /// Services to show logs for
    pub services: Vec<String>,

    /// Follow log output
    #[arg(short = 'f', long)]
    pub follow: bool,

    /// Number of lines to show from the end of the logs
    #[arg(short = 'n', long, default_value = "all")]
    pub tail: String,

    /// Show timestamps
    #[arg(short = 't', long)]
    pub timestamps: bool,

    /// Produce monochrome output
    #[arg(long)]
    pub no_color: bool,

    /// Don't print prefix in logs
    #[arg(long)]
    pub no_log_prefix: bool,

    /// Show logs since timestamp
    #[arg(long)]
    pub since: Option<String>,

    /// Show logs before a timestamp
    #[arg(long)]
    pub until: Option<String>,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,
}

pub async fn run(global: GlobalOpts, args: LogsArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let services: Vec<String> = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };

    let instance_names: Vec<String> = services
        .iter()
        .map(|s| format!("{}_{}_1", project_name, s))
        .collect();

    driver_logs::stream_logs(
        &apptainer,
        &instance_names,
        args.follow,
        args.no_log_prefix,
        args.timestamps,
    )
    .await?;

    Ok(())
}
