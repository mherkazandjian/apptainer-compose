use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct ImagesArgs {
    /// Services to list images for
    pub services: Vec<String>,

    /// Format the output
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Only display IDs
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

pub async fn run(global: GlobalOpts, args: ImagesArgs) -> Result<()> {
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let state = ProjectState::load_or_default(&project_dir, &project_name);

    if !args.quiet {
        println!("{:<20} {:<30} {:<40}", "SERVICE", "IMAGE", "SIF PATH");
    }

    for (service_name, service) in &compose.services {
        if !args.services.is_empty() && !args.services.contains(service_name) {
            continue;
        }

        let image = service.image.as_deref().unwrap_or("-");
        let sif_path = state
            .services
            .get(service_name)
            .and_then(|s| s.instances.first())
            .map(|i| i.image_path.as_str())
            .unwrap_or("-");

        if args.quiet {
            println!("{sif_path}");
        } else {
            println!("{:<20} {:<30} {:<40}", service_name, image, sif_path);
        }
    }

    Ok(())
}
