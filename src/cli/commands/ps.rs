use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::state::project::ProjectState;

#[derive(Args)]
pub struct PsArgs {
    /// Services to list
    pub services: Vec<String>,

    /// Show all stopped containers
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Format output using a custom template
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Don't truncate output
    #[arg(long)]
    pub no_trunc: bool,

    /// Only display IDs
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Display services
    #[arg(long)]
    pub services_flag: bool,

    /// Filter services by status
    #[arg(long)]
    pub status: Vec<String>,

    /// Filter services by a property
    #[arg(long)]
    pub filter: Option<String>,

    /// Include orphaned services
    #[arg(long, default_value_t = true)]
    pub orphans: bool,
}

pub async fn run(global: GlobalOpts, args: PsArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let state = ProjectState::load_or_default(&project_dir, &project_name);
    let live_instances = apptainer.instance_list().await?;

    if args.quiet {
        for (service_name, svc_state) in &state.services {
            if !args.services.is_empty() && !args.services.contains(service_name) {
                continue;
            }
            for inst in &svc_state.instances {
                println!("{}", inst.instance_name);
            }
        }
        return Ok(());
    }

    println!(
        "{:<20} {:<15} {:<30} {:<10}",
        "NAME", "SERVICE", "IMAGE", "STATUS"
    );

    for (service_name, _service) in &compose.services {
        if !args.services.is_empty() && !args.services.contains(service_name) {
            continue;
        }

        let instance_name = format!("{}_{}_1", project_name, service_name);
        let is_live = live_instances.iter().any(|i| i.name == instance_name);
        let status = if is_live { "running" } else { "exited" };

        if !args.all && !is_live {
            continue;
        }

        let image = state
            .services
            .get(service_name)
            .and_then(|s| s.instances.first())
            .map(|i| i.image_source.as_str())
            .unwrap_or("-");

        println!(
            "{:<20} {:<15} {:<30} {:<10}",
            instance_name, service_name, image, status
        );
    }

    Ok(())
}
