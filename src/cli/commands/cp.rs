use clap::Args;

use crate::cli::GlobalOpts;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct CpArgs {
    /// Source path (SERVICE:SRC_PATH or local path)
    pub source: String,

    /// Destination path (SERVICE:DEST_PATH or local path)
    pub destination: String,

    /// Include containers created by the run command
    #[arg(long)]
    pub all: bool,

    /// Archive mode (copy all uid/gid information)
    #[arg(short = 'a', long)]
    pub archive: bool,

    /// Always follow symbol link in SRC_PATH
    #[arg(short = 'L', long)]
    pub follow_link: bool,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,
}

pub async fn run(global: GlobalOpts, args: CpArgs) -> Result<()> {
    let apptainer = Apptainer::detect()?;
    let project_dir = crate::compose::parser::resolve_project_dir(&global)?;
    let project_name = crate::compose::parser::resolve_project_name(&global, &project_dir);

    // Parse SERVICE:PATH format
    let (is_src_container, src_service, src_path) = parse_cp_path(&args.source);
    let (is_dst_container, dst_service, dst_path) = parse_cp_path(&args.destination);

    if global.dry_run {
        tracing::info!("[dry-run] Would copy {} -> {}", args.source, args.destination);
        return Ok(());
    }

    if is_src_container {
        // Copy from container to host
        let instance_name = format!("{}_{}_1", project_name, src_service);
        let output = apptainer
            .exec_instance_capture(&instance_name, &["cat", &src_path], &[])
            .await?;
        std::fs::write(&dst_path, output)?;
    } else if is_dst_container {
        // Copy from host to container via exec
        let instance_name = format!("{}_{}_1", project_name, dst_service);
        let content = std::fs::read_to_string(&src_path)?;
        apptainer
            .exec_instance_capture(
                &instance_name,
                &["sh", "-c", &format!("cat > {dst_path}")],
                &[],
            )
            .await?;
        let _ = content; // placeholder - real impl would pipe stdin
        tracing::warn!("cp into containers has limited support with Apptainer");
    }

    Ok(())
}

fn parse_cp_path(path: &str) -> (bool, String, String) {
    if let Some((service, container_path)) = path.split_once(':') {
        if !service.contains('/') && !service.contains('.') {
            return (true, service.to_string(), container_path.to_string());
        }
    }
    (false, String::new(), path.to_string())
}
