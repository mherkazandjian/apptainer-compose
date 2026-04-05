use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct WaitArgs {
    /// Services to wait for
    pub services: Vec<String>,

    /// Drops project when the first container stops
    #[arg(long)]
    pub down_project: bool,
}

pub async fn run(global: GlobalOpts, args: WaitArgs) -> Result<()> {
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

    tracing::info!("Waiting for services to stop...");

    loop {
        let live = apptainer.instance_list().await?;
        let still_running = instance_names
            .iter()
            .any(|name| live.iter().any(|i| &i.name == name));

        if !still_running {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    tracing::info!("All services have stopped");
    Ok(())
}
