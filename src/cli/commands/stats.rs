use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct StatsArgs {
    /// Service to show stats for
    pub service: Option<String>,

    /// Show all containers
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Format output
    #[arg(long)]
    pub format: Option<String>,

    /// Disable streaming stats
    #[arg(long)]
    pub no_stream: bool,

    /// Do not truncate output
    #[arg(long)]
    pub no_trunc: bool,
}

pub async fn run(global: GlobalOpts, args: StatsArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let services: Vec<String> = if let Some(ref svc) = args.service {
        vec![svc.clone()]
    } else {
        compose.services.keys().cloned().collect()
    };

    for service_name in &services {
        let instance_name = format!("{}_{}_1", project_name, service_name);
        let output = apptainer.instance_stats(&instance_name).await?;
        println!("{service_name}:");
        println!("{output}");
    }

    Ok(())
}
