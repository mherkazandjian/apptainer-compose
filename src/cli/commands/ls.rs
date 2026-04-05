use clap::Args;

use crate::cli::GlobalOpts;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct LsArgs {
    /// Show all stopped Compose projects
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Filter output based on conditions provided
    #[arg(long)]
    pub filter: Option<String>,

    /// Format the output
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Only display project names
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

pub async fn run(global: GlobalOpts, _args: LsArgs) -> Result<()> {
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let state = ProjectState::load_or_default(&project_dir, &project_name);

    let running_count = state
        .services
        .values()
        .filter(|s| s.instances.iter().any(|i| i.status == "running"))
        .count();

    println!(
        "{:<30} {:<10} {:<20}",
        "NAME", "STATUS", "CONFIG FILES"
    );
    let status = if running_count > 0 {
        format!("running({running_count})")
    } else {
        "exited".to_string()
    };
    let config_files = state.compose_files.join(", ");
    println!("{:<30} {:<10} {:<20}", project_name, status, config_files);

    Ok(())
}
