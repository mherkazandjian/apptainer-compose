use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct VolumesArgs {
    /// Services to show volumes for
    pub services: Vec<String>,

    /// Format the output
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Only display volume names
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

pub async fn run(global: GlobalOpts, args: VolumesArgs) -> Result<()> {
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let state = ProjectState::load_or_default(&project_dir, &project_name);

    if !args.quiet {
        println!("{:<30} {:<50}", "VOLUME", "HOST PATH");
    }

    if let Some(ref volumes) = compose.volumes {
        for vol_name in volumes.keys() {
            let host_path = state
                .volumes
                .get(vol_name)
                .map(|v| v.host_path.as_str())
                .unwrap_or("-");

            if args.quiet {
                println!("{vol_name}");
            } else {
                println!("{:<30} {:<50}", vol_name, host_path);
            }
        }
    }

    Ok(())
}
