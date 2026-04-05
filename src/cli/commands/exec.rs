use clap::Args;

use crate::cli::GlobalOpts;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct ExecArgs {
    /// Service name
    pub service: String,

    /// Command to execute
    pub command: Vec<String>,

    /// Detached mode: Run command in the background
    #[arg(short = 'd', long)]
    pub detach: bool,

    /// Set environment variables
    #[arg(short = 'e', long)]
    pub env: Vec<String>,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,

    /// Disable pseudo-TTY allocation
    #[arg(short = 'T', long = "no-tty")]
    pub no_tty: bool,

    /// Give extended privileges to the process
    #[arg(long)]
    pub privileged: bool,

    /// Run the command as this user
    #[arg(short = 'u', long)]
    pub user: Option<String>,

    /// Path to workdir directory for this command
    #[arg(short = 'w', long)]
    pub workdir: Option<String>,
}

pub async fn run(global: GlobalOpts, args: ExecArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let instance_name = format!("{}_{}_1", project_name, args.service);

    if global.dry_run {
        tracing::info!("[dry-run] Would exec in {instance_name}: {:?}", args.command);
        return Ok(());
    }

    let mut extra_args = Vec::new();
    for env_var in &args.env {
        extra_args.push("--env".to_string());
        extra_args.push(env_var.clone());
    }
    if let Some(ref cwd) = args.workdir {
        extra_args.push("--cwd".to_string());
        extra_args.push(cwd.clone());
    }

    let cmd_refs: Vec<&str> = args.command.iter().map(|s| s.as_str()).collect();
    let status = apptainer
        .exec_instance(&instance_name, &cmd_refs, &extra_args)
        .await?;

    std::process::exit(status);
}
