use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct TopArgs {
    /// Services to show processes for
    pub services: Vec<String>,
}

pub async fn run(global: GlobalOpts, args: TopArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let compose = load_compose(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let services: Vec<String> = if args.services.is_empty() {
        compose.services.keys().cloned().collect()
    } else {
        args.services.clone()
    };

    for service_name in &services {
        let instance_name = format!("{}_{}_1", project_name, service_name);
        println!("{service_name}:");

        let output = apptainer
            .exec_instance_capture(&instance_name, &["ps", "-ef"], &[])
            .await?;
        println!("{output}");
    }

    Ok(())
}
