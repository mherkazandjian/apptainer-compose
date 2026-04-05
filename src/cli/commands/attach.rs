use clap::Args;

use crate::cli::GlobalOpts;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct AttachArgs {
    /// Service name
    pub service: String,

    /// Override the key sequence for detaching from a container
    #[arg(long)]
    pub detach_keys: Option<String>,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,

    /// Do not attach STDIN
    #[arg(long)]
    pub no_stdin: bool,

    /// Proxy all received signals to the process
    #[arg(long, default_value_t = true)]
    pub sig_proxy: bool,
}

pub async fn run(global: GlobalOpts, args: AttachArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    let instance_name = format!("{}_{}_1", project_name, args.service);

    // Attach by opening a shell into the instance
    let status = apptainer
        .exec_instance(&instance_name, &["sh"], &[])
        .await?;

    std::process::exit(status);
}
